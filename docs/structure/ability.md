# ability 模块结构

## 模块定位

`src/ability` 提供 GXL 执行时可调用的能力实现（命令、模板、读取、下载、补丁等）。

## 文件结构

```text
src/ability/
├── mod.rs
├── archive.rs
├── assert.rs
├── cmd.rs
├── delegate.rs
├── echo.rs
├── gxl.rs
├── load.rs
├── prelude.rs
├── shell.rs
├── tpl.rs
├── version.rs
├── patch/
│   ├── mod.rs
│   ├── controller.rs
│   ├── model.rs
│   └── view.rs
└── read/
    ├── mod.rs
    ├── cmd.rs
    ├── file.rs
    ├── integra.rs
    └── stdin.rs
```

## 对外导出（`src/ability/mod.rs`）

- 模块：`archive, assert, cmd, delegate, echo, gxl, load, patch, prelude, read, shell, tpl, version`
- 常用类型重导出：`GxAssert, GxCmd, GxEcho, GxRead, GxTpl, TplDTO, GxlSn, GxlVersion, SnAction, GxRun, GxDownLoad, GxUpLoad`（及各自的 Builder）

## 说明

- `ability::patch` 为 `gx.patch_file` 等补丁能力的 MVC 组织。
- 原 `ability::ai` 已下线并整体隔离到 `experimental/ai/`；`src/ability/mod.rs` 中保留注释掉的 `pub mod ai;` 作为恢复锚点，见 `experimental/ai/README.md`。
- `src/ability/mod.rs` 里还有一行注释掉的 `//pub mod vault;`，但仓库中并无 `vault.rs`；而 `src/parser/stc_blk.rs` 仍能解析 `gx.vault`，因此该调用会在装配阶段报 `call not found: gx.vault`。
