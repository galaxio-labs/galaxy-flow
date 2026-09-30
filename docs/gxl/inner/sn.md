# gx.sn

## 作用

读取或更新数字编号文件，并导出编号变量。

## 语法

```gxl
gx.sn(file: "./sn.txt");
gx.sn(file: "./sn.txt", action: "add");
gx.sn(file: "./sn.txt", action: "reset");
gx.sn(file: "./sn.txt", export: "BUILD_SN", action: "add");
```

说明：
- 默认 `action` 为读取：读取文件中的编号，导出变量，不写回文件。
- `action: "add"`：读取当前编号，写回并导出 `当前编号 + 1`。
- `action: "reset"`：写回并导出 `1`。编号文件不存在时也可创建。
- 默认导出变量为 `SN`，可用 `export` 改名。
- 编号文件内容必须是大于等于 `1` 的整数。
