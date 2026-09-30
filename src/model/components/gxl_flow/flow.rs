use crate::ability::prelude::TaskValue;
use crate::components::gxl_env::env::anns_from_option_dto;
use crate::components::gxl_mod::meta::ModMeta;
use crate::components::gxl_spc::GxlSpace;
use crate::components::gxl_utls::mod_obj_name;
use crate::data::AnnDto;
use crate::evaluator::VarParser;
use crate::model::components::prelude::*;

use crate::annotation::{ComUsage, Dryrunable, FST_ARG_TAG, GetArgValue, TaskMessage, Transaction};
use crate::execution::runnable::AsyncRunnableWithSenderTrait;
use crate::execution::task::Task;
use crate::task_report::task_notification::TaskNotice;
use crate::task_report::task_rc_config::{TaskUrlType, build_task_url, report_enable};
use crate::task_report::task_result_report::TaskReport;
use crate::traits::DependTrait;

use crate::components::gxl_block::BlockNode;
use crate::util::http_handle::{create_and_send_task_notice, send_http_request};
use crate::util::redirect::{ReadSignal, init_redirect_file, read_log_content, seek_log_file_end};
use contracts::requires;
use derive_getters::Getters;
use orion_error::conversion::ToStructError;
use std::io::Write;
use std::sync::{Arc, Mutex, mpsc};

use super::anno::FlowAnnFunc;
use super::meta::{FlowMeta, FlowMetaHold};

#[derive(Clone, Getters, Default)]
pub struct GxlFlow {
    meta: FlowMeta,
    blocks: Vec<BlockNode>,
    assembled: bool,
}
impl GxlFlow {
    pub fn meta_mut(&mut self) -> &mut FlowMeta {
        &mut self.meta
    }

    pub(crate) fn bind(&mut self, mod_meta: ModMeta) {
        self.meta.set_host(mod_meta);
    }
    pub fn with_mod(mut self, meta: ModMeta) -> Self {
        self.meta.set_host(meta);
        self
    }
    pub fn with_code(mut self, block: BlockNode) -> Self {
        self.blocks.push(block);
        self
    }
}

impl DependTrait<&GxlSpace> for GxlFlow {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        debug!(target : "assemble", "will assemble flow {}" , self.meta().name() );
        let mut target = GxlFlow::from(self.meta().clone());
        let pre_order_flows = self.meta.preorder();
        let mut buffer = Vec::new();
        let mut linked = false;
        for flow_id in pre_order_flows {
            let meta = assemble_flow_meta(mod_name, flow_id, src)?;
            target.meta_mut().pre_metas_mut().push(meta);
            let _ = write!(&mut buffer, "{flow_id} | ");
            linked = true;
        }

        let _ = write!(&mut buffer, "@{}.{} ", mod_name, self.meta().name());
        let post_order_flows = self.meta.postorder();
        for flow_name in post_order_flows {
            let meta = assemble_flow_meta(mod_name, flow_name, src)?;
            target.meta_mut().pos_metas_mut().push(meta);
            let _ = write!(&mut buffer, " | {flow_name} ");
            linked = true;
        }
        if linked {
            info!(
                target: "assemble",
                "assemble flow  {} ",
                String::from_utf8(buffer).unwrap()
            );
        }
        if let Some(undo_name) = self.meta().undo_flow_name() {
            info!( target: "assemble", "undo flow {undo_name} " );
            let meta = assemble_flow_meta(mod_name, undo_name.as_str(), src)?;
            target.meta_mut().set_undo(meta);
        }
        if let Some(dryrun_name) = self.meta().dryrun_flow_name() {
            info!( target: "assemble", "dryrun flow {dryrun_name} " );
            let meta = assemble_flow_meta(mod_name, dryrun_name.as_str(), src)?;
            target.meta_mut().set_dryrun(meta);
        }
        for block in self.blocks {
            let full_block = block.assemble(mod_name, src)?;
            target.append(full_block);
        }
        target.assembled = true;
        debug!(target : "assemble", "assemble flow {} end" , target.meta().name() );
        Ok(target)
    }
}

fn assemble_flow_meta(m_name: &str, flow: &str, src: &GxlSpace) -> AResult<FlowMeta> {
    let (t_mod, flow_name) = mod_obj_name(m_name, flow);
    debug!(target:"assemble", " find flow by {t_mod}.{flow_name}" );
    if let Some(flow) = src.get(&t_mod).and_then(|m| m.load_scope_flow(&flow_name)) {
        debug!(target:"assemble", "found flow by {t_mod}.{flow_name}" );
        return Ok(flow.meta.clone());
    }
    Err(AssembleError::from(AssembleReason::Miss(format!(
        "{t_mod}.{flow_name}"
    ))))
}

impl From<FlowMeta> for GxlFlow {
    fn from(meta: FlowMeta) -> Self {
        Self {
            meta,
            ..Default::default()
        }
    }
}

impl From<&str> for GxlFlow {
    fn from(name: &str) -> Self {
        let meta = FlowMeta::build_flow(name);
        Self {
            meta,
            ..Default::default()
        }
    }
}

impl GxlFlow {
    pub fn set_anns(&mut self, dto: Option<AnnDto>) {
        self.meta.set_annotates(anns_from_option_dto(dto));
    }
    pub fn load_ins<S: Into<String>>(name: S) -> Self {
        Self {
            meta: FlowMeta::build_flow(name.into()),
            ..Default::default()
        }
    }
}

impl Dryrunable for GxlFlow {
    fn dryrun_hold(&self) -> Option<FlowMetaHold> {
        self.meta().dryrun_meta().clone()
    }
}

impl Transaction for GxlFlow {
    fn is_transaction(&self) -> bool {
        for ann in self.meta().annotations() {
            trace!("flow ann : {ann:?}",);
            if ann.func == FlowAnnFunc::Transaction {
                debug!("flow have transaction lable");
                return true;
            }
        }
        false
    }

    fn undo_hold(&self) -> Option<FlowMetaHold> {
        self.meta().undo_meta().clone()
    }
}
impl GxlFlow {
    #[requires(self.assembled )]
    async fn exec_self(
        &self,
        ctx: ExecContext,
        var_dict: VarSpace,
        sender: Option<mpsc::Sender<ReadSignal>>,
    ) -> TaskResult {
        let task_description = self.task_description(&var_dict);
        let mut task = Task::from(self.meta.name());
        let mut task_notice = TaskNotice::new();

        // 初始化任务并发送通知
        self.init_task_and_notice(&mut task, &mut task_notice, task_description.clone())
            .await?;

        //  通知上层收集日志
        self.notify_log_collection(sender.clone()).await?;

        // 执行所有块
        let (var_dict, task) = self
            .execute_blocks(
                ctx,
                var_dict,
                task_description.clone(),
                task.clone(),
                task_notice.clone(),
            )
            .await?;

        // 完成日志收集
        self.finalize_log_collection(sender).await?;

        // 返回结果
        match task_description {
            Some(_) => Ok(TaskValue::from((var_dict, ExecOut::Task(task)))),
            None => Ok(TaskValue::from((var_dict, ExecOut::Ignore))),
        }
    }

    /// 初始化任务并发送通知
    async fn init_task_and_notice(
        &self,
        task: &mut Task,
        task_notice: &mut TaskNotice,
        task_description: Option<String>,
    ) -> ExecResult<()> {
        if let Some(des) = task_description {
            *task = Task::from(des.clone());
            *task_notice = create_and_send_task_notice(task, task_notice).await?;
        }
        Ok(())
    }

    /// 通知上层收集日志
    async fn notify_log_collection(
        &self,
        sender: Option<mpsc::Sender<ReadSignal>>,
    ) -> ExecResult<()> {
        if let Some(send) = sender {
            let log_file = init_redirect_file()?;
            let end_pos = seek_log_file_end(&log_file)?;
            send.send(ReadSignal::Start(end_pos)).map_err(|e| {
                ExecReason::Io
                    .to_err()
                    .with_detail(format!("flow send task error: {e}"))
            })?;
        }
        Ok(())
    }

    // 辅助方法：执行所有块
    async fn execute_blocks(
        &self,
        ctx: ExecContext,
        mut var_dict: VarSpace,
        task_description: Option<String>,
        mut task: Task,
        task_notice: TaskNotice,
    ) -> Result<(VarSpace, Task), ExecError> {
        for item in &self.blocks {
            // 执行需要重定向日志的块
            if task_description.is_some() && report_enable().await {
                let (var_dict_new, task_new) = self
                    .execute_block_with_monitoring(
                        item,
                        ctx.clone(),
                        var_dict,
                        task_description.clone(),
                        task,
                        task_notice.clone(),
                    )
                    .await?;
                var_dict = var_dict_new;
                task = task_new;
            } else {
                let TaskValue { vars, rec, .. } =
                    item.async_exec(ctx.clone(), var_dict, None).await?;
                var_dict = vars;
                task.append(rec);
            }
        }
        task.finish();
        Ok((var_dict, task))
    }

    // 辅助方法：执行单个块并监控日志
    async fn execute_block_with_monitoring(
        &self,
        block: &BlockNode,
        ctx: ExecContext,
        var_dict: VarSpace,
        task_description: Option<String>,
        mut task: Task,
        task_notice: TaskNotice,
    ) -> Result<(VarSpace, Task), ExecError> {
        let (cur_sender, receiver) = mpsc::channel::<ReadSignal>();
        let log_file = init_redirect_file()?;
        let start_pos = Arc::new(Mutex::new(seek_log_file_end(&log_file)?));

        let shared_output = Arc::new(Mutex::new(String::new()));
        let shared_output_clone = Arc::clone(&shared_output);
        let mut shared_task = Arc::new(task.clone());
        let share_task_notice = Arc::new(task_notice.clone());
        let start_pos_clone = Arc::clone(&start_pos);
        let monitor_handle: tokio::task::JoinHandle<ExecResult<()>> = tokio::spawn(async move {
            while let Ok(flag) = receiver.recv() {
                match flag {
                    ReadSignal::Start(end) => {
                        let start = {
                            let guard = start_pos_clone.lock().map_err(|e| {
                                ExecReason::system_error()
                                    .to_err()
                                    .with_detail(format!("lock task log start position: {e}"))
                            })?;
                            *guard
                        };
                        let buf = read_log_content(&log_file, start, end).await?;
                        if let Some(task_ref) = Arc::get_mut(&mut shared_task) {
                            task_ref.stdout.push_str(&buf);
                        }

                        let url = build_task_url(TaskUrlType::TaskReport)
                            .await
                            .unwrap_or_default();
                        let task_result = {
                            TaskReport::from_task_and_notice(
                                (*shared_task).clone(),
                                (*share_task_notice).clone(),
                            )
                        };
                        if let Ok(mut data) = shared_output_clone.lock() {
                            data.push_str(&buf);
                        }
                        send_http_request(task_result.clone(), &url).await;
                    }
                    ReadSignal::End(cur_start) => {
                        let mut guard = start_pos_clone.lock().map_err(|e| {
                            ExecReason::system_error()
                                .to_err()
                                .with_detail(format!("lock task log start position: {e}"))
                        })?;
                        *guard = cur_start;
                    }
                }
            }
            Ok(())
        });
        let sender_option = task_description.as_ref().map(|_| cur_sender.clone());
        let TaskValue { vars, .. } = match block.async_exec(ctx, var_dict, sender_option).await {
            Ok(TaskValue { vars, rec }) => {
                task.clone().append(rec);
                Self::update_task_with_output(&mut task, &shared_output, start_pos).await?;

                if task_description.is_some() {
                    self.report_task_status(&task, &task_notice).await?;
                }
                TaskValue::from((vars, ExecOut::Task(task.clone())))
            }
            Err(e) => {
                task.result = Err(e.to_string());
                Self::update_task_with_output(&mut task, &shared_output, start_pos).await?;
                if task_description.is_some() {
                    self.report_task_status(&task, &task_notice).await?;
                }
                return Err(ExecError::from(e));
            }
        };
        drop(cur_sender);
        drop(monitor_handle);
        Ok((vars, task))
    }

    /// 报告任务状态
    async fn report_task_status(&self, task: &Task, task_notice: &TaskNotice) -> ExecResult<()> {
        let url = build_task_url(TaskUrlType::TaskReport)
            .await
            .unwrap_or_default();
        let task_result = TaskReport::from_task_and_notice(task.clone(), task_notice.clone());
        send_http_request(task_result, &url).await;
        Ok(())
    }

    /// 完成日志收集
    async fn finalize_log_collection(
        &self,
        sender: Option<mpsc::Sender<ReadSignal>>,
    ) -> ExecResult<()> {
        if let Some(send) = sender {
            let log_file = init_redirect_file()?;
            let end_pos = seek_log_file_end(&log_file)?;
            send.send(ReadSignal::End(end_pos)).map_err(|e| {
                ExecReason::Io
                    .to_err()
                    .with_detail(format!("flow send task error: {e}"))
            })?;
        }
        Ok(())
    }

    // 更新任务输出
    async fn update_task_with_output(
        task: &mut Task,
        shared_output: &Arc<Mutex<String>>,
        start_pos: Arc<Mutex<u64>>,
    ) -> ExecResult<()> {
        if let Ok(output) = shared_output.lock()
            && !output.is_empty()
        {
            task.stdout = output.clone();
        }

        let log_path = init_redirect_file()?;
        let end_pos = seek_log_file_end(&log_path)?;
        let start = *start_pos.lock().map_err(|e| {
            ExecReason::system_error()
                .to_err()
                .with_detail(format!("lock task log start position: {e}"))
        })?;
        let content = read_log_content(&log_path, start, end_pos).await?;
        task.stdout.push_str(&content);

        Ok(())
    }

    // 获取注解中的描述信息
    pub fn get_desan(&self) -> Option<String> {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.desp().is_some() {
                return ann.desp();
            }
        }
        None
    }

    // 获取注解中的描述信息
    pub fn task_description(&self, var_dict: &VarSpace) -> Option<String> {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.message().is_some() {
                let exp = EnvExpress::from_env_mix(var_dict.global().clone());
                return ann.message().map(|msg| exp.safe_eval(msg.as_str()));
            }
        }
        None
    }

    pub fn is_dryrun(&self) -> bool {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.func == FlowAnnFunc::Dryrun {
                return true;
            }
        }
        false
    }
    pub fn is_auto_entry(&self) -> bool {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.func == FlowAnnFunc::AutoLoad
                && ann.get_arg(FST_ARG_TAG) == Some("entry".to_string())
            {
                return true;
            }
        }
        false
    }
    pub fn is_auto_exit(&self) -> bool {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.func == FlowAnnFunc::AutoLoad
                && ann.get_arg(FST_ARG_TAG) == Some("exit".to_string())
            {
                return true;
            }
        }
        false
    }
}
#[async_trait]
impl AsyncRunnableWithSenderTrait for GxlFlow {
    async fn async_exec(
        &self,
        mut ctx: ExecContext,
        mut var_dict: VarSpace,
        sender: Option<mpsc::Sender<ReadSignal>>,
    ) -> TaskResult {
        let des = self
            .task_description(&var_dict)
            .unwrap_or(self.meta.name().to_string());
        let mut job = Job::from(&des);
        ctx.append(self.meta.name());
        let TaskValue { vars, rec, .. } = self.exec_self(ctx.clone(), var_dict, sender).await?;
        var_dict = vars;
        job.append(rec);
        Ok(TaskValue::from((var_dict, ExecOut::Job(job))))
    }
}
impl ComponentMeta for GxlFlow {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::Flow(self.meta.clone())
    }
}

impl AppendAble<BlockNode> for GxlFlow {
    fn append(&mut self, block: BlockNode) {
        self.blocks.push(block);
    }
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::{components::GxlMod, infra::once_init_log, meta::MetaInfo};

    use super::*;

    #[test]
    fn test_assemble_com_without_dependencies() {
        // 创建一个 RgMod 实例
        let gxl_mod = GxlMod::from("test_mod");

        // 创建一个目标 RgFlow 实例，没有依赖关系
        let target_flow = GxlFlow::from("target_flow");

        let mut spc = GxlSpace::default();
        spc.append(gxl_mod);
        // 调用 assemble_com 方法
        let assembled_flow = target_flow.assemble("test_mod", &spc).assert();

        // 验证 pre_ows 和 post_ows 是否为空
        assert_eq!(assembled_flow.meta().pre_metas().len(), 0);
        assert_eq!(assembled_flow.meta().pos_metas().len(), 0);
    }

    #[test]
    fn test_assemble_com_with_missing_dependencies() {
        once_init_log();
        // 创建一个 RgMod 实例，包含部分依赖关系
        let mut gxl_mod = GxlMod::from("test_mod");

        // 创建一些 RgFlow 实例
        let flow1 = GxlFlow::from("flow1");
        let flow2 = GxlFlow::from("flow2");

        // 将这些 RgFlow 实例添加到 RgMod 中
        gxl_mod.append(flow1);
        gxl_mod.append(flow2);

        // 创建一个目标 RgFlow 实例，包含部分存在的依赖关系
        let mut target_flow = GxlFlow::from("target_flow");
        target_flow.meta.set_preorder(vec!["flow1", "non_existent"]);
        target_flow
            .meta
            .set_postorder(vec!["flow2", "non_existent2"]);

        gxl_mod.append(target_flow);
        let mut spc = GxlSpace::default();
        spc.append(gxl_mod);
        assert!(spc.assemble().is_err());
        // 调用 assemble_com 方法
        //assert!(target_flow.assemble("test_mod", &spc).is_err());
    }

    #[test]
    fn test_assemble_com_with_multiple_dependencies() {
        // 创建一个 RgMod 实例，包含多个依赖关系
        let mut gxl_mod = GxlMod::from("test_mod");

        // 创建一些 RgFlow 实例
        let flow1 = GxlFlow::from(FlowMeta::build_flow("flow1".to_string()));
        let flow2 = GxlFlow::from(FlowMeta::build_flow("flow2".to_string()));
        let flow3 = GxlFlow::from(FlowMeta::build_flow("flow3".to_string()));

        // 将这些 RgFlow 实例添加到 RgMod 中
        gxl_mod.append(flow1);
        gxl_mod.append(flow2);
        gxl_mod.append(flow3);

        // 创建一个目标 RgFlow 实例，包含多个依赖关系
        let mut target_flow = GxlFlow::from("target_flow");
        target_flow.meta.set_preorder(vec!["flow1", "flow2"]);
        target_flow.meta.set_postorder(vec!["flow3"]);
        //gxl_mod.append(target_flow.clone());
        let mut spc = GxlSpace::default();
        spc.append(gxl_mod);
        spc = spc.assemble().assert();

        // 调用 assemble_com 方法
        let assembled_flow = target_flow.assemble("test_mod", &spc).assert();

        // 验证 pre_ows 和 post_ows 是否正确组装
        assert_eq!(assembled_flow.meta().pre_metas().len(), 2);
        assert_eq!(
            assembled_flow.meta().pre_metas()[0].long_name(),
            "test_mod.flow1"
        );
        assert_eq!(
            assembled_flow.meta().pre_metas()[1].long_name(),
            "test_mod.flow2"
        );

        assert_eq!(assembled_flow.meta().pos_metas().len(), 1);
        assert_eq!(
            assembled_flow.meta().pos_metas()[0].long_name(),
            "test_mod.flow3"
        );
    }
}
