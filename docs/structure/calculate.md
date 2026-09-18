# calculate 模块结构

## 模块定位

`src/calculate` 负责表达式求值、比较、逻辑组合和条件判定。

## 实际文件结构

```text
src/calculate/
├── mod.rs
├── compare.rs
├── cond.rs
├── defined.rs
├── dynval.rs
├── express.rs
├── logic.rs
└── traits.rs
```

## 对外导出（`src/calculate/mod.rs`）

- 模块：`compare, cond, defined, dynval, express, logic, traits`
- 类型：`CmpExpress, ExpressEnum, Evaluation`
