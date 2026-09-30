# Galaxy Flow 使用指南

本文档基于以下资料整理：
- `docs/structure/*`
- `docs/gxl/*`

目标：给使用者一份从“项目初始化 -> 编写 GXL -> 执行与发布”的实操入口。

## 1. 核心概念

Galaxy Flow 现在以 `gx` 作为统一 CLI 入口：
- `gx run`：执行工作流（默认 `./_gal/work.gxl`）
- `gx adm`：执行管理流（默认 `./_gal/adm.gxl`）
- `gx init/mod/doc/check/self`：项目与工具链管理

GXL 的核心结构：
- `mod`：模块
- `env`：环境上下文
- `flow`：流程入口与编排单元
- `fn`：可复用函数
- `activity`：可复用活动定义

## 2. 目录与运行模型

典型项目目录：
- `./_gal/work.gxl`：工作流配置（给 `gx run`）
- `./_gal/adm.gxl`：管理流配置（给 `gx adm`）
- `./_gal/mods/`：本地模块（可选）
- `updates/{stable|alpha|beta}/manifest.json`：自更新清单

运行链路（从结构文档抽象）：
1. `parser` 将 GXL 文本解析为模型
2. `model` 承载执行对象
3. `evaluator/runner` 调度执行
4. `ability` 提供 `gx.*` 内置能力

## 3. 快速开始

### 3.1 初始化环境与项目

```bash
# 初始化运行环境（首次使用）
gx init env

# 初始化项目（本地，不依赖远程模板）
gx init project

# 初始化项目（从默认模板仓库的子目录）
# 默认仓库：https://github.com/galaxio-labs/prj-tpl.git
gx init project --path rust

# 初始化项目（从指定仓库）
gx init project --repo https://your-tpl-repo.git
```

说明：
- `gx init project` 不带 `--repo`/`--path` 时，执行本地初始化，创建基本的 `./_gal/` 目录结构
- `gx init project --path xxx`（或 `--repo xxx`）时，从 git 仓库拉取模板；`--branch` 与 `--tag` 互斥
- 完整参数说明见 `docs/guidle/cli/gx.md`

### 3.2 查看配置与运行

```bash
# 查看工作流信息（不传 flow 时展示信息）
gx run

# 运行指定 flow
gx run conf

# 运行管理流
gx adm conf
```

### 3.3 更新模块

```bash
gx mod update
```

## 4. GXL 写作最小模板

```gxl
mod demo {
  env default {
    root = "${HOME}";
  }

  flow conf {
    gx.echo(value: "hello galaxy flow");
  }
}
```

建议先掌握：
- 变量：`docs/gxl/var_def.md`
- 常量：`docs/gxl/const.md`
- 语法：`docs/gxl/syntax.md`
- 内置能力：`docs/gxl/inner/index.md`

## 5. 常用内置能力

高频能力（详见 `docs/gxl/inner/*.md`）：
- `gx.echo`：打印/输出
- `gx.cmd` / `gx.shell`：执行命令
- `gx.read_*`：从文件/命令/stdin 读取变量
- `gx.tpl`：模板渲染
- `gx.assert`：断言
- `gx.ver`：版本处理
- `gx.patch_file`：基于 marker 做文件补丁

### 5.1 `gx.patch_file` 约束（关键）

- `strict` 语义：任何 marker 结构异常都应失败
- `comment_prefix` 不允许为空字符串
- 支持 marker 形式：`@gxl:*`、`//@gxl:*`、`#@gxl:*`

典型 marker：
- 行替换：`@gxl:set(id)`
- 行注释：`@gxl:line(id)`
- 块注释：`@gxl:block(id)` + `@gxl:end(id)`

## 6. CLI 文档入口

- `gx`：`docs/guidle/cli/gx.md`
- 快速文档：`gx doc gx.cmd`、`gx doc gx.patch_file`

## 7. 自更新工作流（gx self）

当前模型是”手动触发式”：
- 检查更新：`gx self check --channel <stable|alpha|beta>`
- 执行更新：`gx self update --channel <channel> --yes`
- 回滚：`gx self rollback [--id <backup_id>]`

说明：
- 下载包在临时目录，完成后清理
- 最终替换当前安装目录中的 `gx`
- 备份目录：`~/.galaxy/self_update/backups/<backup_id>/`
- Manifest 来源：`galaxio-labs/get` 仓库的 `updates/gx/{channel}/manifest.json`
- 实现基于 `wp-self-update` 库

## 8. 推荐学习路径

1. 先跑通 `gx init` + `gx run conf`
2. 学会 `env + flow + gx.echo/gx.cmd`
3. 加入模板与断言（`gx.tpl`、`gx.assert`）
4. 引入 `gx.patch_file` 做配置变更自动化
5. 使用 `gx self` 管理版本升级与回滚
