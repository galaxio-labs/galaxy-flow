# Galaxy Flow 项目结构

本文档描述当前仓库中的真实结构（以工作树代码为准）。

## Workspace 主体

```text
galaxy-flow/
├── app/
│   └── gx/
│       └── main.rs          # gx CLI 入口
├── src/
│   ├── ability/             # GXL abilities 实现
│   ├── calculate/           # 表达式与条件计算
│   ├── conf/                # 项目配置加载
│   ├── evaluator/           # 环境表达式渲染
│   ├── model/               # 运行时与语法模型
│   ├── parser/              # GXL 语法解析
│   ├── self_update/         # 自升级检查/安装/状态存储
│   ├── templates/           # `gx init project` 的内置模板（include_str! 编译进二进制）
│   └── util/                # 通用工具
├── crates/
│   ├── orion_parse/         # 解析基础能力
│   └── orion_cond/          # 条件表达式能力
├── docs/
├── examples/
├── experimental/            # 已下线能力的隔离代码，不参与编译（见 experimental/ai/README.md）
├── scripts/                 # 仓库脚本（GXL 文档镜像同步等）
└── tests/
```

## `src` 模块导出概览

- `ability`: `archive, assert, cmd, delegate, echo, gxl, load, patch, prelude, read, shell, tpl, version`
- `calculate`: `compare, cond, defined, dynval, express, logic, traits`
- `conf`: `gxlconf, oprator`（`mod_test` 为内部测试模块）
- `evaluator`: 对外仅导出 `EnvExpress, VarParser`（来自 `env_exp.rs`）
- `model`: `annotation, components, context, data, error, execution, expect, meta, primitive, task_report, traits, var`
- `parser`: `abilities, atom, code, cond, context, domain, externs, gxl_fun, inner, prelude, stc_*`
- `self_update`: `mod, model, rollback, service, storage`
- `util`: `accessor, diagnose, git(内部), http_handle, opt, path, redirect, shell, str_utils, task_report` 等

## 对齐文档

- `docs/structure/ability.md`
- `docs/structure/calculate.md`
- `docs/structure/conf.md`
- `docs/structure/evaluator.md`
- `docs/structure/model.md`
- `docs/structure/parser.md`
- `docs/structure/util.md`

## 维护规则

- 本目录每个模块一个文件，直接记录结构事实。
- 注意 `docs/` 下的 Markdown 并非“纯文档”：`src/help.rs` 用 `include_str!` 把
  `docs/guidle/index.md`、`docs/guidle/cli/gx.md` 与 `docs/gxl/**` 编进二进制供 `gx doc` 使用。
- 增删 `src/` 下的模块目录后，请同步更新本目录中的对应文件。
