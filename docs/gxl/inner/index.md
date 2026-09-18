# GXL 内置能力总览（对齐当前实现）

以下为当前 block parser 直接支持的内置能力：

- `gx.assert`：`docs/gxl/inner/assert.md`
- `gx.cmd`：`docs/gxl/inner/cmd.md`
- `gx.echo`：`docs/gxl/inner/echo.md`
- `gx.read_file / gx.read_cmd / gx.read_stdin`：`docs/gxl/inner/read.md`
- `gx.vars`（仅 env 内）：`docs/gxl/inner/vars.md`
- `gx.tpl`：`docs/gxl/inner/tpl.md`
- `gx.ver`：`docs/gxl/inner/ver.md`
- `gx.sn`：`docs/gxl/inner/sn.md`
- `gx.run`：`docs/gxl/inner/run.md`
- `gx.shell`：`docs/gxl/inner/shell.md`
- `gx.tar / gx.untar`：`docs/gxl/inner/tar_untar.md`
- `gx.download / gx.upload`：`docs/gxl/inner/download_upload.md`
- `gx.patch_file`：`docs/gxl/inner/patch_file.md`

表达式函数：
- `defined(${VAR})`：`docs/gxl/inner/defined.md`

兼容别名说明：
- 解析器中部分能力存在 `rg.xxx` 兼容解析入口，但在 block/env 语句分发层并未统一开放。
- 建议始终使用 `gx.xxx`。

未作为当前内置 block 能力接入：
- `gx.artifact`（见 `docs/gxl/inner/artifact.md`）
