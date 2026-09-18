use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::Sender;

use crate::friendly::AppendAble;
use async_trait::async_trait;
use orion_error::conversion::ConvErr;

use crate::ExecResult;
use crate::ability::prelude::TaskValue;
use crate::annotation::{Dryrunable, Transaction};
use crate::components::gxl_flow::meta::FlowMeta;
use crate::components::gxl_spc::GxlSpace;
use crate::context::ExecContext;
use crate::execution::VarSpace;
use crate::execution::hold::AsyncComHold;
use crate::execution::hold::{ComHold, IsolationHold};
use crate::execution::job::Job;
use crate::execution::runnable::{AsyncRunnableTrait, ExecOut, TaskResult};
use crate::execution::runnable::{AsyncRunnableWithSenderTrait, ComponentMeta};
use crate::execution::task::Task;
use crate::execution::trans::ComTrans;
use crate::meta::{GxlMeta, MetaInfo};
use crate::util::redirect::ReadSignal;

use super::hold::TransableHold;
use super::unit::{RunUnitGuard, RunUnitLable};

#[derive(Clone, Getters, Default)]
pub struct ExecSequence {
    name: String,
    filter: UniqueFilter,
    run_items: Vec<ComHold>,
}

#[derive(Clone, Default)]
pub struct UniqueFilter {
    items: HashSet<RunUnitLable>,
}
impl UniqueFilter {
    pub fn is_pass(&mut self, key: &RunUnitLable) -> bool {
        if !self.items.contains(key) {
            self.items.insert(key.clone());
            return true;
        }
        false
    }
}

impl From<&str> for ExecSequence {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

pub trait SequLoader {
    fn find_flow(&self, meta: &FlowMeta, sequ: &mut impl SequAppender) -> ExecResult<()>;
}
pub trait SequAppender {
    fn append_hold<H: Into<TransableHold>>(&mut self, guard: &RunUnitGuard, hold: H);
}

#[derive(Clone, Getters, Default)]
pub struct FlowSequence {
    name: String,
    filter: UniqueFilter,
    run_items: Vec<TransableHold>,
}

pub trait HoldRunable: Dryrunable + AsyncRunnableTrait + Clone {}
impl ExecSequence {
    pub async fn execute(
        &self,
        ctx: ExecContext,
        def: VarSpace,
        spc: &impl SequLoader,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        self.execute_sequence(ctx, def, spc, sender).await
    }

    pub async fn test_execute(
        &self,
        ctx: ExecContext,
        def: VarSpace,
        spc: &GxlSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        self.execute_sequence(ctx, def, spc, sender).await
    }

    async fn execute_sequence(
        &self,
        ctx: ExecContext,
        mut def: VarSpace,
        spc: &impl SequLoader,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        warn!(target: ctx.path(), "sequence size: {}  dryrun: {}", self.run_items().len(), ctx.dryrun());

        let mut trans_manage = ComTrans::new();
        let mut job = Job::from(&self.name);
        for (index, item) in self.run_items.iter().enumerate() {
            info!(target: ctx.path(), "executing item {}: {} ", index, item.gxl_meta().full_name());
            if trans_manage.in_transaction_trigger(item.is_transaction())
                && let Some(undo) = item.undo_hold()
            {
                let mut sequ = ExecSequence::default();
                spc.find_flow(&undo, &mut sequ).conv_err()?;
                for undo in sequ.run_items() {
                    info!(target: ctx.path(), "regist undo {}", undo.gxl_meta().name());
                    trans_manage.add_undo_task(undo.clone(), def.clone());
                }
            }
            match self
                .execute_hold(&ctx, &mut def, spc, item, sender.clone())
                .await
            {
                Ok(TaskValue { vars, rec, .. }) => {
                    def = vars;
                    job.append(rec);
                }
                Err(e) => {
                    warn!("Sequence aborted : {e}");
                    trans_manage.rollback(&ctx).await;
                    return Err(e);
                }
            }
        }

        Ok(TaskValue::from((def, ExecOut::Job(job))))
    }

    async fn execute_hold(
        &self,
        ctx: &ExecContext,
        def: &mut VarSpace,
        spc: &impl SequLoader,
        item: &ComHold,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        let mut job = Job::from(&self.name);
        let mut exec_queue = build_exec_queue(ctx, spc, item)?;
        while let Some(item) = exec_queue.pop_back() {
            let TaskValue { vars, rec, .. } = item
                .async_exec(ctx.clone(), def.clone(), sender.clone())
                .await?;
            *def = vars;
            job.append(rec);
        }
        Ok(TaskValue::new(def.clone(), ExecOut::Job(job)))
    }
}

fn build_exec_queue(
    ctx: &ExecContext,
    spc: &impl SequLoader,
    item: &ComHold,
) -> ExecResult<VecDeque<ComHold>> {
    let mut sub_queue = VecDeque::new();
    if ctx.dryrun()
        && let Some(dryrun_meta) = item.dryrun_hold()
    {
        let mut sequ = ExecSequence::default();
        spc.find_flow(&dryrun_meta, &mut sequ).conv_err()?;
        for dryrun in sequ.run_items() {
            info!(target: ctx.path(), "regist undo {}", dryrun.gxl_meta().name());
            sub_queue.push_back(dryrun.clone());
        }
        return Ok(sub_queue);
    }
    sub_queue.push_back(item.clone());
    Ok(sub_queue)
}

impl SequAppender for ExecSequence {
    fn append_hold<H: Into<TransableHold>>(&mut self, guard: &RunUnitGuard, hold: H) {
        let trans_hold = hold.into();
        debug_assert!(trans_hold.assembled());
        debug!(target:"exec/sque", "only_item :{:?}", guard.lable());
        if self.filter.is_pass(guard.lable()) || *guard.open() {
            info!(target:"exec/sque", "only_item :{}", trans_hold.gxl_meta().full_name());
            self.run_items.push(AsyncComHold::from(trans_hold).into());
        }
    }
}
impl SequAppender for FlowSequence {
    fn append_hold<H: Into<TransableHold>>(&mut self, guard: &RunUnitGuard, hold: H) {
        let trans_hold = hold.into();
        debug_assert!(trans_hold.assembled());
        debug!(target:"exec/sque", "only_item :{:?}", guard.lable());
        if self.filter.is_pass(guard.lable()) || *guard.open() {
            info!(target:"exec/sque", "only_item :{}", trans_hold.gxl_meta().full_name());
            self.run_items.push(trans_hold);
        }
    }
}

impl AppendAble<AsyncComHold> for ExecSequence {
    fn append(&mut self, node: AsyncComHold) {
        debug!(target: "exec/sque", "append {}", node.gxl_meta().full_name() );
        self.run_items.push(node.into());
    }
}

impl AppendAble<IsolationHold> for ExecSequence {
    fn append(&mut self, node: IsolationHold) {
        debug!(target: "exec/sque", "append {}", node.hold().gxl_meta().full_name() );
        self.run_items.push(node.into());
    }
}

#[derive(Clone, Default)]
pub struct StubAction {
    name: String,
}

impl StubAction {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

impl ComponentMeta for StubAction {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("stub")
    }
}

#[async_trait]
impl AsyncRunnableTrait for StubAction {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult {
        //TODO:
        debug!(target: ctx.path(), "executing stub: {}", self.name);
        let task = Task::from(&self.name);
        Ok(TaskValue::from((def, ExecOut::Task(task))))
    }
}
