# model 模块结构

## 模块定位

`src/model` 定义 GXL 的核心数据结构、执行模型和任务报告模型。

## 实际文件结构

```text
src/model/
├── mod.rs
├── annotation.rs
├── context.rs
├── data.rs
├── error.rs
├── expect.rs
├── meta.rs
├── primitive.rs
├── traits.rs
├── var.rs
├── components/
│   ├── mod.rs
│   ├── prelude.rs
│   ├── gxl_block.rs
│   ├── gxl_cond.rs
│   ├── gxl_extend.rs
│   ├── gxl_intercept.rs
│   ├── gxl_loop.rs
│   ├── gxl_prop.rs
│   ├── gxl_spc.rs
│   ├── gxl_utls.rs
│   ├── gxl_var.rs
│   ├── gxl_act/{mod,meta,activity}.rs
│   ├── gxl_env/{mod,meta,anno,env}.rs
│   ├── gxl_flow/{mod,meta,anno,flow,runner}.rs
│   ├── gxl_fun/{mod,meta,fun}.rs
│   └── gxl_mod/{mod,meta,anno,body}.rs
├── execution/
│   ├── mod.rs
│   ├── action.rs
│   ├── dict.rs
│   ├── global.rs
│   ├── hold.rs
│   ├── job.rs
│   ├── runnable.rs
│   ├── sequence.rs
│   ├── task.rs
│   ├── trans.rs
│   └── unit.rs
└── task_report/
    ├── mod.rs
    ├── main_task.rs
    ├── task_notification.rs
    ├── task_rc_config.rs
    └── task_result_report.rs
```

## 对外导出（`src/model/mod.rs`）

- 模块：`annotation, components, context, data, error, execution, expect, meta, primitive, task_report, traits, var`
- 类型：`ExecError, ExecReason, ExecResult`

## 说明

- `components/gxl_intercept.rs` 文件存在，但在 `components/mod.rs` 中当前未导出。
- 模块中不存在 `sec.rs`。
