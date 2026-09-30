# experimental/ai — 已隔离的 AI 能力代码

本目录**不属于任何 crate，不会被编译**。它保存 galaxy-flow 早期 AI 能力的实现，等 AI 能力恢复时再迁回。

## 为什么在这里

`orion-ai` 依赖在 `Cargo.toml` 中被注释后，AI 模块的 `pub mod` 声明也被注释掉，于是这些文件既不参与编译、也没有被删除——属于「在树里但从不编译」的死代码，会无声腐化（一旦有人取消注释就会立刻编译失败）。为了既保住这份实现、又不再误导读者，统一隔离到这里。

隔离时的实证结论：整个 `src/ability/ai/` 模块（含 `tool.rs` 与三个常量 `AI_CONTENT` / `AI_CALL_RESULT` / `AI_CALL_VALUE`）**没有任何存活的消费方**，常量也只被本目录内的死代码引用。

## 目录结构

目录结构 1:1 镜像原始路径，恢复时按路径搬回即可：

| 隔离位置 | 原始路径 |
|---|---|
| `src/ability/ai/` | `src/ability/ai/` |
| `src/parser/inner/ai_chat.rs` | `src/parser/inner/ai_chat.rs` |
| `src/parser/inner/ai_regst.rs` | `src/parser/inner/ai_regst.rs` |
| `src/parser/inner/ai_task.rs` | `src/parser/inner/ai_task.rs` |
| `AI_DESIGN.md` | `src/AI_DESIGN.md` |

## 恢复步骤

1. 在 `Cargo.toml` 恢复 `orion-ai` 依赖（文件里保留了 git / path 两种备选写法）。
2. 搬回源码：

   ```bash
   git mv experimental/ai/src/ability/ai src/ability/ai
   git mv experimental/ai/src/parser/inner/ai_chat.rs src/parser/inner/ai_chat.rs
   git mv experimental/ai/src/parser/inner/ai_regst.rs src/parser/inner/ai_regst.rs
   git mv experimental/ai/src/parser/inner/ai_task.rs src/parser/inner/ai_task.rs
   ```

3. 还原模块声明：`src/ability/mod.rs` 的 `pub mod ai;`，以及 `src/parser/inner/mod.rs`、`src/ability/ai/mod.rs` 中被注释掉的 `pub mod` / `pub use`。
4. `ai_diagnose`（`src/util/diagnose.rs`）当前是 no-op，需一并接回。
5. 跑 `cargo clippy --all-targets --all-features -- -D warnings` 与
   `cargo test --all-features -- --test-threads=1`。

## 注意

- 本目录内的代码**未经编译验证**，迁移时很可能需要按当时的 `orion-ai` API 调整。
- `AI_DESIGN.md` 描述的是当初的设计（其中提到的 `src/ai/` 目录从未落地），不要当作现状描述。
- 相关文档：`examples/AI_CONFIG_USAGE.md`、`examples/ai_fun_basic/README.md`、
  `tasks/ai-workflow-engine-integration/` 中的示例与计划同样处于下线状态。
