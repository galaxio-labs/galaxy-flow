# gx 命令使用文档

`gx` 是 Galaxy Flow 的统一命令入口。

它将原来的工作流执行、管理流执行、项目初始化、模块更新、文档查看、自更新等能力收敛到一个命令下。

## 基本用法

```bash
gx <COMMAND>
```

## 命令总览

- `run`：执行工作流，默认配置 `./_gal/work.gxl`
- `adm`：执行管理流，默认配置 `./_gal/adm.gxl`
- `init`：初始化环境或项目
- `mod`：项目模块管理
- `doc`：快速查看 GXL/CLI 文档主题
- `check`：检查当前运行环境
- `self`：自更新（status/check/update/rollback）

## 常用示例

```bash
# 初始化运行环境（首次使用）
gx init env

# 初始化项目（本地基础模板，无需网络）
gx init project

# 初始化项目（从默认仓库子目录，需要网络）
gx init project --path rust

# 初始化项目（从指定仓库）
gx init project --repo https://github.com/user/repo.git
gx init project --repo https://github.com/user/repo.git --branch dev

# 运行工作流
gx run conf

# 仅判断流程是否存在（不执行）：存在 exit 0，不存在 exit 1
gx run localize --exists

# 运行管理流
gx adm conf

# 查看帮助主题
gx doc
gx doc gx.cmd
gx doc --markdown gx.cmd

# 更新模块
gx mod update
```

## init project 详解

```bash
# 本地初始化（离线、快速，创建基础 work.gxl 和 adm.gxl）
gx init project

# 从默认仓库子目录初始化（需要网络）
# 默认仓库: https://github.com/galaxio-labs/prj-tpl.git
gx init project --path rust
gx init project --path rust --branch main

# 从指定仓库初始化
gx init project --repo <git_url>
gx init project --repo <git_url> --branch <branch>
gx init project --repo <git_url> --tag <tag>

# 从指定仓库子目录初始化
gx init project --repo <git_url> --path <subdir>
```

参数说明：
- `--repo`：git 仓库地址
- `--path`：仓库内的子目录路径（使用默认仓库或指定仓库）
- `--branch`：指定分支（需要 `--repo` 或 `--path`）
- `--tag`：指定标签（需要 `--repo` 或 `--path`，与 `--branch` 互斥）

行为说明：
- 不带参数 → 本地初始化，创建基础配置文件
- 带 `--path`（无 `--repo`）→ 从默认仓库的指定子目录初始化
- 带 `--repo` → 从指定仓库初始化

## 兼容入口

- 如果做了链接，`grun ...` 等价于 `gx run ...`
- 如果做了链接，`gadm ...` 等价于 `gx adm ...`

## 目录约定

- `./_gal/work.gxl`：默认工作流入口，给 `gx run`
- `./_gal/adm.gxl`：默认管理流入口，给 `gx adm`
- `./_gal/mods/`：项目本地模块目录

## 文档阅读建议

- 想看 CLI 总览：`gx doc gx`
- 想看能力细节：`gx doc gx.cmd`、`gx doc gx.patch_file`
