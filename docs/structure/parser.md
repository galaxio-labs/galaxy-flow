# parser 模块结构

## 模块定位

`src/parser` 负责把 GXL 源码解析为 `model` 层结构。

## 实际文件结构

```text
src/parser/
├── mod.rs
├── atom.rs
├── cond.rs
├── context.rs
├── domain.rs
├── externs.rs
├── prelude.rs
├── stc_act.rs
├── stc_ann.rs
├── stc_base.rs
├── stc_blk.rs
├── stc_env.rs
├── stc_mod.rs
├── stc_spc.rs
├── abilities/
│   ├── mod.rs
│   ├── addr.rs
│   ├── comment.rs
│   ├── define.rs
│   ├── param.rs
│   └── prelude.rs
├── gxl_fun/
│   ├── mod.rs
│   ├── head.rs
│   └── body.rs
├── inner/
│   ├── mod.rs
│   ├── archive.rs
│   ├── assert.rs
│   ├── call.rs
│   ├── cmd.rs
│   ├── common.rs
│   ├── funs.rs
│   ├── gxl.rs
│   ├── load.rs
│   ├── patch.rs
│   ├── read.rs
│   ├── shell.rs
│   ├── tpl.rs
│   ├── ver.rs
│   └── prelude.rs
├── stc_flow/
│   ├── mod.rs
│   ├── head.rs
│   └── body.rs
└── code/            # 解析样例/测试资源，不由 mod.rs 导出
```

## 对外导出（`src/parser/mod.rs`）

- `atom, domain, externs, abilities, cond, context, gxl_fun, inner, prelude`
- `stc_act, stc_ann, stc_base, stc_blk, stc_env, stc_flow, stc_mod, stc_spc`

## 说明

- `inner/mod.rs` 中 AI 解析文件当前未导出（已注释）。
