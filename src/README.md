# src/

`galaxy_flow` 库 crate 的全部实现。运行时主链路：

`parser`（解析 GXL 文本）→ `model`（执行模型）→ `ability`（`gx.*` 内置能力）→ `runner` / `evaluator`（调度与表达式求值）

- CLI 入口与命令定义：`cli.rs`、`cmd/`
- 编译进二进制的文档：`help.rs`（用 `include_str!` 引用 `docs/` 下的 Markdown）
- 模块结构说明：`../docs/structure/project-structure.md`
- 仓库协作约定：`../AGENTS.md`
- 已下线能力的隔离代码：`../experimental/`
