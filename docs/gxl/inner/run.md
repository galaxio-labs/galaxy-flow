# gx.run

## 作用

在子目录中执行另一个 GXL 配置。

## 语法

```gxl
gx.run(
  local: "<run dir>",
  conf: "<gxl file>",
  env: "<env name>",
  flow: "a,b,c",
  isolate: "true|false"
);
```

参数：
- `local`：运行目录
- `conf`：目标配置文件（默认 `./_gal/work.gxl`）
- `env`：目标环境
- `isolate`：是否隔离变量空间
- `flow`：转发的目标流程；逗号分隔可指定多个（逐個转发）。**必填**，且 `local`/`env` 需一并给出。
