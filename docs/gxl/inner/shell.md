# gx.shell

## 作用

执行 shell 命令/脚本，可加载参数文件，可回填输出变量。

对于执行时间较长的脚本，可使用 `stream: "true"` 将输出实时显示到当前会话。

## 语法

```gxl
gx.shell(
  shell: "<command or script>",
  arg_file: "<json|yml|yaml|toml|ini>",
  out_var: "<var name>",
  err: "<err var>",
  ok_codes: "0,2",
  log: "1|2|3",
  sudo: "true|false",
  silence: "true|false",
  stream: "true|false"
);
```

也支持匿名首参数：

```gxl
gx.shell("./demo.sh");
```

## 示例

```gxl
gx.shell(
  shell: "PYTHONUNBUFFERED=1 ./deploy.sh",
  out_var: "DEPLOY_OUT"
);

gx.shell(
  shell: "PYTHONUNBUFFERED=1 ansible-playbook site.yml -vv",
  stream: "true"
);
```

## `stream` 说明

- `stream: "true"` 时，脚本执行中的 stdout/stderr 会实时输出到当前会话
- 实时输出不影响 `out_var`、退出码和内部结果记录
- 如果脚本本身有缓冲，建议同时设置对应程序的无缓冲参数
