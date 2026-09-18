# gx.read_file / gx.read_cmd / gx.read_stdin

## gx.read_file

读取配置文件到变量空间。

```gxl
gx.read_file(
  file: "./var.yml",
  name: "DATA"
);
```

参数：
- `file`（或匿名首参数）
- `name`：可选；不传时对象字段会并入全局变量
- `entity`：解析层接受，当前执行层未使用

格式支持：
- `ini`
- `json`
- `yml`

## gx.read_cmd

执行命令并把 stdout 写入变量。

```gxl
gx.read_cmd(
  name: "BRANCH",
  cmd: "git branch --show-current",
  err: "ERR_MSG",
  ok_codes: "0,1",
  log: "1",
  stream: "true|false"
);
```

说明：

- `gx.read_cmd` 只会把 `stdout` 写入变量
- `stream: "true"` 时，命令执行中的 stdout/stderr 会实时输出到当前会话
- 即使开启 `stream`，最终写入变量的仍然只是 stdout

## gx.read_stdin

从标准输入读取值。

```gxl
gx.read_stdin(
  prompt: "input your name:",
  name: "USER_NAME"
);
```
