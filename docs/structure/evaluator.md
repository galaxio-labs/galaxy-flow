# evaluator 模块结构

## 模块定位

`src/evaluator` 当前只承载环境表达式渲染能力，不是独立的流程执行器目录。

## 实际文件结构

```text
src/evaluator/
├── mod.rs
└── env_exp.rs
```

## 对外导出（`src/evaluator/mod.rs`）

- `EnvExpress`
- `VarParser`
