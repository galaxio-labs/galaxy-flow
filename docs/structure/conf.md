# conf 模块结构

## 模块定位

`src/conf` 负责 `.gxlprj` 配置的定位、初始化和加载。

## 实际文件结构

```text
src/conf/
├── mod.rs
├── gxlconf.rs
├── oprator.rs
└── mod_test.rs
```

## 对外导出（`src/conf/mod.rs`）

- 模块：`gxlconf, oprator`
- 函数：`conf_init, conf_path, load_gxl_config`

## 说明

- `mod_test.rs` 由 `mod.rs` 内部引用，仅用于测试。
