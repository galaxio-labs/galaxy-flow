# util 模块结构

## 模块定位

`src/util` 提供 git/路径/shell/http 等通用支撑能力。

## 实际文件结构

```text
src/util/
├── mod.rs
├── accessor.rs
├── diagnose.rs
├── git.rs
├── http_handle.rs
├── macs.rs
├── opt.rs
├── path.rs
├── redirect.rs
├── serialize_time_format.rs
├── shell.rs
├── str_utils.rs
├── task_report.rs
└── traits.rs
```

## 对外导出（`src/util/mod.rs`）

- `GitTools`（来自内部模块 `git`）
- `os_sh`（来自 `shell`）
- `OptionFrom`（来自 `opt`）
- 公共模块：`http_handle, path, serialize_time_format, shell, str_utils, task_report, traits, opt, accessor, diagnose, redirect`

## 说明

- `git` 与 `macs` 在 `mod.rs` 中为内部模块（非 `pub mod`）。
- 模块中不存在 `cache.rs`、`collection.rs`、`crypto.rs` 等文件。
