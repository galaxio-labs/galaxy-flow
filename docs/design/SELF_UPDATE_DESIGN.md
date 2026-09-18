# 自动升级能力设计

- 文档状态：已实现
- 最后更新：2026-03-21

## 1. 概述

`gx self` 提供自更新能力，支持检查、升级、回滚操作。

**实现方案**：基于 `wp-self-update` 库（当前 `Cargo.toml` 为 `wp-self-update = "0.3"`），manifest 存储在独立的 `galaxio-labs/get` 仓库。

## 2. CLI 命令

```bash
gx self status
gx self check --channel <stable|alpha|beta> [--json]
gx self update --channel <stable|alpha|beta> [--to <version>] [--dry-run] [--force] --yes
gx self rollback [--id <backup_id>]
```

说明：
- `status`：显示当前版本、安装路径、最近检查结果
- `check`：检查远端是否有新版本
- `update`：下载、校验、替换、健康检查；失败自动回滚
- `rollback`：回滚到最近一次或指定备份

## 3. 配置与状态文件

存储位置：`~/.galaxy/self_update/`

```
~/.galaxy/self_update/
  state.json     # 状态记录
  lock           # 并发锁
  backups/       # 备份目录
    <backup_id>/
      gx         # 备份的二进制
```

## 4. 发布通道

| Channel | 说明 | Manifest 路径 |
|---------|------|---------------|
| stable | 稳定发布 | `updates/gx/stable/manifest.json` |
| alpha | Alpha 预发布 | `updates/gx/alpha/manifest.json` |
| beta | Beta 预发布 | `updates/gx/beta/manifest.json` |

## 5. Manifest 规范

Manifest 存储在 `galaxio-labs/get` 仓库，由 CI 自动生成。

URL 格式：
```
https://raw.githubusercontent.com/galaxio-labs/get/main/updates/gx/{channel}/manifest.json
```

Manifest 结构：
```json
{
  "version": "0.13.5",
  "channel": "stable",
  "published_at": "2026-03-21T10:00:00Z",
  "git_ref": "v0.13.5",
  "git_commit": "abcdef...",
  "assets": {
    "aarch64-apple-darwin": {
      "url": "https://github.com/galaxio-labs/galaxy-flow/releases/download/v0.13.5/galaxy-flow-v0.13.5-aarch64-apple-darwin.tar.gz",
      "sha256": "..."
    },
    "x86_64-unknown-linux-gnu": { ... },
    "x86_64-unknown-linux-musl": { ... }
  }
}
```

## 6. 升级流程

1. 读取当前版本：`env!("CARGO_PKG_VERSION")`
2. 拉取指定 channel 的 manifest
3. 比较 semver，确定是否需要更新
4. 下载目标 tar.gz 到临时目录
5. 校验 sha256
6. 解压并验证包含 `gx`
7. 获取 lock（防并发）
8. 备份当前 bin 到 `backups/<timestamp>/`
9. 原子替换
10. 健康检查：`gx --version`
11. 成功写 `state.json`；失败自动回滚

## 7. 安全策略

- 强制 sha256 校验
- 仅允许 GitHub Releases 域名
- 并发锁 + 回滚闭环
- 包管理器安装检测（`/usr/local/bin`、`/usr/bin` 等），需 `--force` 覆盖

## 8. CI/CD 集成

Release workflow (`.github/workflows/release.yml`) 包含 `update-gx-get-manifest` job：

1. 构建产物发布到 GitHub Releases
2. 拉取 release 元数据
3. 计算 sha256
4. 生成 manifest.json
5. 推送到 `galaxio-labs/get` 仓库的 `updates/gx/{channel}/` 目录

需要配置 GitHub Secret：`GX_GET_TOKEN`（有 `galaxio-labs/get` 仓库写入权限的 PAT）。

> 说明：release workflow 通过 `GX_GET_TOKEN` checkout `galaxio-labs/get` 并写入上述目录，
> 与 `src/self_update/service.rs` 的读取地址一致。

## 9. 代码结构

```
src/self_update/
  mod.rs        # 模块导出
  model.rs      # 类型定义（CheckResult, UpdateResult 等）
  service.rs    # 核心逻辑（check/update/rollback）
  storage.rs    # 状态存储
  rollback.rs   # 回滚与健康检查
```

依赖：`wp-self-update = "0.3"`

## 10. 使用示例

```bash
# 检查当前状态
gx self status

# 检查 alpha 通道更新
gx self check --channel alpha

# 更新到最新 stable（需确认）
gx self update --channel stable --yes

# 强制更新（即使版本不更高）
gx self update --channel alpha --force --yes

# 回滚到上一个版本
gx self rollback
```
