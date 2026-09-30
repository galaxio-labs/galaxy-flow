# gx.download / gx.upload

## gx.download

```gxl
gx.download(
  url: "https://example.com/a.txt",
  local_file: "./temp/a.txt",
  username: "user",
  password: "pass",
  force: "true"
);
```

## gx.upload

```gxl
gx.upload(
  url: "https://example.com/upload",
  local_file: "./temp/a.txt",
  method: "put",
  username: "user",
  password: "pass"
);
```

说明：
- `local_file` 的父目录必须已存在。
- `gx.download` 若 `local_file` 是目录，会按 URL 文件名落盘。
- `force`（可选，默认 `"false"`）：本地文件已存在时默认跳过（`reuse_cache`）；
  `force: "true"` 忽略本地文件、强制重下（走 `UpdateScope::RemoteCache`，会清理该地址的缓存）。
- 下载中断不会在目标路径留下半包（先写 `<local_file>.part`，成功后原子替换；失败时清掉临时文件）。
- 传输失败（网络抖动 / 5xx / 截断）内部最多重试 3 次；4xx 与本地 IO 错误不重试。
