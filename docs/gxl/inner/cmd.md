# gx.cmd

## 作用

执行一条命令字符串。

默认会在命令结束后统一输出结果；当命令执行时间较长且需要实时观察输出时，可以启用 `stream: "true"`。

## 语法

```gxl
gx.cmd(
  cmd: "<command>",
  err: "<err var>",
  suc: "<success message>",
  ok_codes: "0,2",
  sudo: "true|false",
  log: "1|2|3",
  silence: "true|false",
  quiet: "true|false",
  stream: "true|false"
);
```

也支持匿名首参数：

```gxl
gx.cmd("echo hello");
```

## 代码块命令

````gxl
```cmd
cp a b
ls -al
```
````

## 示例

```gxl
gx.cmd(cmd: "git branch --show-current");
gx.cmd("echo ${HOME}");
gx.cmd(cmd: "grep foo missing.txt", ok_codes: "0,1");
gx.cmd(cmd: "ansible-playbook site.yml -vv", stream: "true");
```

## `stream` 说明

- `stream: "false"`：默认行为，命令完成后再统一打印并返回 stdout/stderr
- `stream: "true"`：执行期间把 stdout/stderr 实时转发到当前会话，同时仍然保留完整输出用于 `Action` 记录

适合：

- `ansible-playbook`
- `terraform apply`
- `kubectl rollout status`
- 其他执行时间长、需要边跑边看的命令

说明：

- `stream` 不改变退出码判断逻辑，仍由 `ok_codes` 控制
- `stream` 与 `quiet: "true"` 同时使用时，不会实时打印到终端，但仍会走流式读取并保留输出
- 终端上 stdout/stderr 的实时交错顺序不保证稳定；如果业务上必须固定顺序，需要在 shell 层自行合并流
