use std::collections::VecDeque;

use crate::{
    ability::prelude::ComponentMeta, context::ExecContext,
    execution::runnable::AsyncRunnableWithSenderTrait,
};

use super::{VarSpace, hold::ComHold};

// 事务管理器，跟踪事务状态和撤销任务
#[derive(Clone, Default)]
pub struct TransactionManager<T>
where
    T: AsyncRunnableWithSenderTrait,
{
    in_transaction: bool,
    undo_stack: VecDeque<(T, VarSpace)>, // 存储待撤销的任务
}

impl<T> TransactionManager<T>
where
    T: AsyncRunnableWithSenderTrait + ComponentMeta,
{
    pub fn new() -> Self {
        Self {
            in_transaction: false,
            undo_stack: VecDeque::new(),
        }
    }

    pub fn begin_transaction(&mut self) {
        warn!(target: "trans", "transaction begin");
        self.in_transaction = true;
    }
    pub fn in_transaction_trigger(&mut self, flag: bool) -> bool {
        if flag {
            self.begin_transaction();
        }
        self.in_transaction
    }

    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }

    pub fn add_undo_task(&mut self, task: T, vars: VarSpace) {
        if self.in_transaction {
            self.undo_stack.push_back((task, vars));
        }
    }

    pub async fn rollback(&mut self, ctx: &ExecContext) {
        while let Some((undo, dict)) = self.undo_stack.pop_back() {
            match undo.async_exec(ctx.clone(), dict, None).await {
                Ok(_) => warn!("Undo successful for {}", undo.gxl_meta().name()),
                Err(e) => error!("Undo failed for {}: {}", undo.gxl_meta().name(), e),
            }
        }
    }
}
pub type ComTrans = TransactionManager<ComHold>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ExecReason,
        ability::prelude::{AsyncRunnableTrait, ExecOut, TaskResult, TaskValue},
        context::ExecContext,
        meta::GxlMeta,
        util::redirect::ReadSignal,
    };
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex, mpsc::Sender};

    // Mock runnable task for testing
    #[derive(Clone)]
    struct MockTask {
        name: String,
        execute_count: Arc<Mutex<u32>>,
        should_fail: bool,
    }

    impl MockTask {
        fn new(name: &str, should_fail: bool) -> Self {
            Self {
                name: name.to_string(),
                execute_count: Arc::new(Mutex::new(0)),
                should_fail,
            }
        }
    }

    #[async_trait]
    impl AsyncRunnableTrait for MockTask {
        async fn async_exec(&self, _ctx: ExecContext, _vars: VarSpace) -> TaskResult {
            let mut count = self.execute_count.lock().unwrap();
            *count += 1;

            if self.should_fail {
                Err(ExecReason::from_logic().into())
            } else {
                Ok(TaskValue::new(_vars, ExecOut::Ignore))
            }
        }
    }

    #[async_trait]
    impl AsyncRunnableWithSenderTrait for MockTask {
        async fn async_exec(
            &self,
            _ctx: ExecContext,
            _vars: VarSpace,
            _sender: Option<Sender<ReadSignal>>,
        ) -> TaskResult {
            let mut count = self.execute_count.lock().unwrap();
            *count += 1;

            if self.should_fail {
                Err(ExecReason::from_logic().into())
            } else {
                Ok(TaskValue::new(_vars, ExecOut::Ignore))
            }
        }
    }

    impl ComponentMeta for MockTask {
        fn gxl_meta(&self) -> GxlMeta {
            GxlMeta::Simple(format!("meta:{}", self.name))
        }
    }

    #[tokio::test]
    async fn test_transaction_lifecycle() {
        let mut manager = TransactionManager::new();

        // Not in transaction - task shouldn't be added
        let task = MockTask::new("test_task", false);
        manager.add_undo_task(task.clone(), VarSpace::default());
        assert_eq!(manager.undo_stack.len(), 0);

        // Begin transaction
        manager.begin_transaction();
        assert!(manager.in_transaction);

        // Add task during transaction
        manager.add_undo_task(task.clone(), VarSpace::default());
        assert_eq!(manager.undo_stack.len(), 1);
    }

    #[tokio::test]
    async fn test_rollback_execution_order() {
        let mut manager = TransactionManager::new();
        manager.begin_transaction();

        let ctx = ExecContext::default();
        let task1 = MockTask::new("task1", false);
        let task2 = MockTask::new("task2", false);
        let task3 = MockTask::new("task3", false);

        manager.add_undo_task(task1.clone(), VarSpace::default());
        manager.add_undo_task(task2.clone(), VarSpace::default());
        manager.add_undo_task(task3.clone(), VarSpace::default());

        // Rollback should execute in reverse order
        manager.rollback(&ctx).await;

        assert_eq!(*task1.execute_count.lock().unwrap(), 1);
        assert_eq!(*task2.execute_count.lock().unwrap(), 1);
        assert_eq!(*task3.execute_count.lock().unwrap(), 1);
        assert!(manager.undo_stack.is_empty());
    }

    #[tokio::test]
    async fn test_rollback_error_handling() {
        let mut manager = TransactionManager::new();
        manager.begin_transaction();

        let ctx = ExecContext::default();
        let good_task = MockTask::new("good", false);
        let bad_task = MockTask::new("bad", true);

        manager.add_undo_task(good_task.clone(), VarSpace::default());
        manager.add_undo_task(bad_task.clone(), VarSpace::default());

        // Rollback should execute both tasks even if one fails
        manager.rollback(&ctx).await;

        assert_eq!(*good_task.execute_count.lock().unwrap(), 1);
        assert_eq!(*bad_task.execute_count.lock().unwrap(), 1);
    }
}
