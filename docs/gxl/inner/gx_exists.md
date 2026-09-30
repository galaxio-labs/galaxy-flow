# gx.exists(...)

## 作用

条件表达式函数：判断**流程**是否存在。用于「有则调用、无则跳过」，与 `defined(${VAR})`
同一层次——直接写在 `if` 里。

判定依据是**当前已装配空间的流程名集合**，因此：

- **不解析 extern、不联网、无副作用**；
- 不存在返回 `false`（**不是错误**），不依赖任何退出码。

## 语法

```gxl
if gx.exists(flow: "localize") {
    gx.run(local: ".", env: "default", flow: "localize");
}
```

参数：

- `flow`：流程名。也接受裸值 `gx.exists("localize")` 或变量 `gx.exists(${P})`。
- 未限定名默认归属 `main` 模块；也接受 `mod.flow` 限定名（如 `gx.exists("ops-mod.install")`）。

## 示例

```gxl
mod main {
    flow deploy {
        if gx.exists(flow: "precheck") {
            gx.run(local: ".", env: "default", flow: "precheck");
        } else {
            gx.echo(value: "skip precheck");
        }

        # 变量形式 + 取反
        if !gx.exists(${OPTIONAL_FLOW}) {
            gx.echo(value: "optional flow not defined");
        }
    }
}
```

## 与命令行的关系

命令行侧提供等价能力 `gx run <flow> --exists`：存在退出码 `0`，不存在退出码 `1`。
它适合在 **gx 之外**（脚本或上层工具，例如 `galaxy-ops`）判断某个流程是否存在后再决定是否调用。

几点说明：

- 判定只关心 conf 声明的流程，**与 `-e/--env` 无关**；
- `--exists` 会**加载 conf（但不执行任何流程）**；conf 无法加载（含 extern 模块未缓存）时按「不存在」处理，退出 `1`，原因打到 stderr。
