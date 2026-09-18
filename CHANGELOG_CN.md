# 更新日志

Galaxy Flow项目所有重要变更将记录在此文件中。

本格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/) 标准，
本项目遵循 [语义化版本2.0.0](https://semver.org/lang/zh-CN/spec/v2.0.0.html) 规范。

## [v0.13.15] - 2026-09-18

### 新增
- **GXL 文档镜像工具**：新增 `scripts/sync-gxl-docs.sh`（`sync` / `check` / `list`），用于生成并校验 `operator-docs` 中的 `docs/gxl/` 副本。镜像边界是显式的：`docs/gxl/example/*.md` 归 `operator-docs` 自有，永不被覆盖。

### 文档
- **`gx init project` 参数修正**：README 与使用指南改为记录真实参数（`--repo` / `--path` / `--branch` / `--tag`），去掉并不存在的 `--tpl`；并说明不带 `--repo`/`--path` 时为本地初始化（`--branch` 与 `--tag` 互斥）。
- **README 的 AI 状态**：`--ai` 标注为当前无效（`ai_diagnose` 是 no-op）；`gx run` 参数列表补上原先遗漏的 `--log` 与 `-q/--quiet`。
- **GXL 语法文档去重**：移除过时的重复文档 `docs/syntax.md`；`docs/gxl/syntax.md`（同时也是 `gx doc gxl` 渲染的内容）成为唯一语法文档，并新增注解章节、补上内置能力列表中遗漏的 `gx.sn`。
- **结构文档与设计文档对齐实现**：把重复的 `docs/structure/*-actual.md` 合并进 `docs/structure/*.md`，并更正与代码不符的描述——`self_update` 文件列表、已删除的初始化模板目录、`wp-self-update = "0.3"`、release manifest 的仓库名。
- **协作者指南重写**：`AGENTS.md` 此前描述的是已不存在的 `gflow` / `gprj` 双二进制布局；现改为记录单二进制 `gx`、真实的构建/测试/clippy 命令（含 `-D warnings`），以及 `docs/` 下的 Markdown 会经 `include_str!` 编译进二进制这一事实。

### 变更
- **Release manifest 仓库名统一**：`.github/workflows/release.yml` 改为 checkout 并推送到 `galaxio-labs/get`，补齐此前 `galaxy-sec` → `galaxio-labs` 的迁移。

### 修复
- **解析器健壮性**：`extern_parse` 中 `DslStatus::Data` 原为 `todo!()`，现改为返回错误而不是 panic。
- **新工具链下的 clippy**：去掉 `debug!` / `info!` / `format!` 参数中多余的引用——rustc 1.98+ 的 `clippy::useless_borrows_in_formatting` 会拒绝（CI 以 `-D warnings` 跑 clippy）。

### 移除
- **仓库内安装脚本**：移除 `install.sh` 的安装逻辑及其读取的 `updates/{stable,alpha,beta}/manifest.json` 清单。该链路长期未维护（stable 停在 `0.12.4`，sha256 是全零占位符，安装时会静默跳过校验），且与 `gx self` 使用的清单来源不是同一份数据。`install.sh` 现为弃用提示桩，退出并指向官方安装脚本（`https://get.warpparse.ai/inst-x.sh`）；`gx self` 仍读取 `galaxio-labs/get`。
- **已被取代的初始化模板**：移除 `app/gx/init/_gal/`。自初始化模板迁移到 `src/templates/`（用 `include_str!` 编译进二进制）之后，`gx init project` 一直从那里生成脚手架，旧目录没有任何引用。
- **未参与编译的 AI 代码**：把被注释掉、从不参与编译的 AI 实现（`src/ability/ai/*`、`src/parser/inner/ai_*.rs`、`src/AI_DESIGN.md`）移到 `experimental/ai/` 并附恢复说明。`galaxy_flow::ability::ai` 不再属于库的公开 API。

## [v0.13.14] - 2026-05-04

### 新增
- **GXL 数字编号能力**：新增 `gx.sn`，用于读取和更新简单数字编号文件。默认动作只读取当前编号并导出为 `SN`，不写回文件；`action: "add"` 会递增并写回下一个编号；`action: "reset"` 会写回并导出 `1`。
- **文档主题**：新增 `gx doc gx.sn`，并将 `gx.sn` 加入内置文档主题列表和 GXL 内置能力索引。

## [v0.13.13] - 2026-05-04

### 变更
- **依赖升级**：orion-error 升级至 0.8，orion_conf 至 0.7，orion-sec 至 0.6，orion-infra 至 0.7，orion-variate 至 0.13，orion-accessor 至 0.8。
- **结构化错误模型**：将 `RunReason`、`ExecReason`、`GxlReason` 的业务错误变体迁移为 unit enum，动态诊断文本统一放到 `StructError` 的 detail/source/context 中，不再放入 enum payload。
- **orion-error v0.8 source 链处理**：在 IO、HTTP、模板目录遍历、JSON 序列化、UTF-8 解码等边界，用 `source_err` 或 `source_raw_err` 替换部分 `map_err`，保留底层 source 供错误报告使用。

### 移除
- **旧错误兼容路径**：继续清理 `.owe()` 时代的兼容写法，统一使用当前 orion-error 0.8 API。

## [v0.13.12] - 2026-04-29

### 变更
- **依赖升级**: orion-error 升级至 0.7, orion_conf 至 0.6, orion-sec 至 0.5, orion-infra 至 0.6, orion-variate 至 0.12, orion-accessor 至 0.7。

- **orion-error v0.7 API 迁移**: 全代码库适配新版本 API 变更：
  - `WithContext::want()` → `WithContext::doing()`, `.with()` → `.with_context()` 上下文错误链重命名。
  - 模块导入路径调整：`ErrorOwe`、`ErrorOweBase` 移至 `orion_error::compat_traits`；`ToStructError`、`ContextRecord` 移至 `orion_error::traits_ext`。
  - `OperationContext::want()` → `doing()` 及 `.with()` → `.with_context()`（tpl、archive、load、version 等能力模块）。
  - 自定义错误枚举 `RunReason` 与 `GxlReason` 新增 `DomainReason` trait 实现。
  - 测试工具迁移：`TestAssertWithMsg` → `orion_error::testcase::TestAssertWithMsg`，单元测试中新增 `TestAssert` 使用。
  - 全面清理已废弃的 `ErrorOwe` 用法，替换为 `ErrorOweBase` + `UvsReason`，消除 80+ 弃用警告。
  - 用 `.owe(UvsReason::*.into())` 替代分类简写方法 `.owe_*()`，错误分类显式化。

## [v0.13.11] - 2026-04-21

### 新增
- **Shell 实时输出能力**：为 `gx.cmd`、`gx.shell`、`gx.read_cmd` 共用的 shell 选项新增 `stream: "true"`，长时间命令执行时可将 stdout/stderr 实时转发到当前会话，同时继续保留命令结果中的输出内容与退出码。

### 文档
- **GXL shell 文档同步**：更新 `docs/gxl` 下的 command、shell、read、syntax、help 与示例文档，补充 `stream` 模式说明、ansible 场景示例，以及实时输出时 stdout/stderr 交错顺序的说明。

### 修复
- **长时间命令可观测性**：`gx.cmd` 在命令确实持续产生输出时，不再要等命令结束后才统一打印，避免 ansible 等长任务执行过程中出现“像卡住了一样”的体验。

## [v0.13.10] - 2026-04-07

### 变更
- **自更新通道默认值**：`gx self check` 与 `gx self update` 的 `--channel` 现在默认取 `stable`，对应主分支发布通道，日常使用不再需要每次显式传参。
- **自更新检查输出**：重做 `gx self check` 的终端输出，改为更清晰的状态报告，明确展示 channel、current、remote 以及版本关系；当存在新版本时会直接给出建议的升级命令。

### 修复
- **远程项目子目录初始化来源**：修复 `gx init project --path <subdir>` 的远程初始化逻辑，确保真正把远程仓库中指定子目录的内容复制到 `./_gal/`，不再因为中间缓存检出路径而落入错误模板内容。
- **初始化失败清理**：远程项目初始化在创建 `./_gal/` 后如果失败，现在会清理整个目标目录，而不再只尝试删除空目录。

## [v0.13.9] - 2026-04-07

### 修复
- **项目初始化子目录扁平化**：`gx init project --path <subdir>` 现在会把选中模板目录的内容直接落到 `./_gal/` 下，不再残留诸如 `./_gal/rust/` 的多余目录层级。
- **初始化目标边界保护**：为项目初始化补充内部路径边界校验，只允许整理 `./_gal/` 内部的下载结果，避免误移动或误删除初始化目录之外的内容。
- **CLI 与 shell 测试隔离**：对 loader、cmd、shell、activity 相关测试补充工作目录固定与稳定路径处理，避免并行测试下因共享 `cwd` 污染导致的随机失败。
- **模块更新配置测试隔离**：将 `gx mod update` 的用户配置回归测试改为在临时项目目录中运行，不再误触仓库内 git extern 模块更新，也避免远程路径导致的慢测试告警。

## [v0.13.8] - 2026-04-06

### 变更
- **仓库归属对齐**：将项目、发布、manifest、安装脚本中的地址从历史的 `galaxy-sec` 统一切换到 `galaxio-labs`。
- **项目初始化默认仓库**：将 `gx init project` 的默认模板仓库及相关 CLI 帮助文本更新为 `https://github.com/galaxio-labs/prj-tpl.git`。
- **自更新来源切换**：将自更新 manifest 和安装脚本的默认来源切换为新的 `galaxio-labs` 仓库路径。

### 文档
- **README 与指南同步**：更新 README、CLI 指南、语法说明、自更新设计文档和 updates 文档，使其与当前仓库归属和发布地址保持一致。
- **发布元数据同步**：同步 `0.13.8` 的版本元数据与 manifest 模板引用。

### 修复
- **示例与测试地址**：更新内置示例、模板和测试中的仓库地址，避免初始化流程、解析示例和模板加载继续指向已停用的 `galaxy-sec` 路径。

## [v0.13.5] - 2026-03-23

### 变更
- **项目初始化 CLI 重构**：将 `--tpl` 替换为拆分后的 `--repo` 与 `--path` 参数。
  - `gx init project`：本地初始化（离线生成基础 `work.gxl` 和 `adm.gxl`）
  - `gx init project --path rust`：从默认仓库的子目录远程初始化
  - `gx init project --repo <url>`：从自定义仓库远程初始化
- **默认仓库**：当只提供 `--path` 而未指定 `--repo` 时，默认使用 `https://github.com/galaxio-labs/prj-tpl.git`。
- **参数校验**：`--branch` 与 `--tag` 现在要求与 `--repo` 或 `--path` 一起使用。

### 修复
- **目录清理**：当内容复制到 `_gal` 失败时，现在会自动清理已创建的空目录。

## [v0.13.4] - 2026-03-15

### 变更
- **命令成功语义**：`gx.cmd`、`gx.shell`、`gx.read_cmd` 的 shell 选项字段由 `expect` 重命名为 `ok_codes`，明确表示“允许视为成功的退出码集合”。
- **动作结果模型**：`Action` 的命令执行结果改为结构化输出 `exit_code`、`stdout`、`stderr`，不再把 stdout/stderr 混写到单一字段中，便于 machine output 消费。
- **模块更新结果输出**：`gx mod update` 现在会按配置文件汇总 git extern modules，并输出最终的项目模块更新结果摘要。

### 修复
- **模块更新配置收集**：当 `./_gal/work.gxl` 与 `./_gal/adm.gxl` 同时存在时，`gx mod update` 现在会同时纳入两者，不再遗漏已有配置。
- **CLI 输出契约**：`gx mod update` 的说明性结果摘要统一输出到 `stderr`，继续保持 `stdout` 不承载 human-oriented 说明文本。
- **命令解析与测试覆盖**：补充 `ok_codes` 的解析与执行回归用例，并校验命令执行结果中的 `exit_code/stdout/stderr` 字段。

## [v0.13.0] - 2026-03-10

### 新增
- **统一 `gx` CLI**：新增 `gx` 作为唯一发布命令入口，统一承载 `run`、`adm`、`init`、`mod`、`doc`、`check`、`self`。
- **短别名入口**：新增 `grun` -> `gx run` 与 `gadm` -> `gx adm` 的别名启动支持。
- **命令内文档阅读**：新增 `gx doc` 主题索引与终端友好的 markdown 渲染，`--markdown` 可输出原始 markdown。

### 变更
- **项目初始化资源**：本地初始化模板目录从 `app/gprj/init` 迁移到 `app/gx/init`。
- **发布链路**：release workflow、安装脚本、自更新流程改为只打包和校验 `gx` 二进制。
- **文档入口**：README 与 guide 文档统一改为 `gx` 视角，不再以旧二进制作为主入口说明。
- **CLI 命令模型**：模块管理统一为 `gx mod update`；`gx run` 与 `gx adm` 拆成独立 clap 包装和独立 help；CLI 文案统一收敛为更明确的 `project` 语义。
- **CLI 运行时初始化**：`gx init project` 改为走与其他命令一致的运行时初始化路径，统一覆盖日志初始化与 `~/.galaxy/conf.toml` 加载。

### 移除
- **旧命令二进制**：移除 `gflow` 与 `gprj` 的工作区二进制定义及发布产物。
- **旧 CLI 文档页**：移除 guide 中独立的 `gflow` / `gprj` 使用文档。
- **旧版 gx 兼容入口**：移除已废弃的 `gx conf`、`gx init prj`、`gx init prj-with-local`、`gx update mod`、`--mod_up`、`--conf-work`、`--conf-adm` 等旧兼容语法与参数。

### 修复
- **仓库自举解析**：将仓库 `_gal/work.gxl` 中的 extern 路径改为 `./_gal/`，避免 `GXL_START_ROOT` 缺失时触发解析失败。
- **模块更新语义**：`gx mod update` 现在会稳定加载 `~/.galaxy/conf.toml`，并在 `./_gal/work.gxl` / `./_gal/adm.gxl` 都不存在时直接报错。
- **CLI 退出行为**：`gx adm` 在所有 flow 都失败时现在返回非零退出码；`gx run` / `gx adm` 无显式 flow 时保留展示菜单能力并返回成功。
- **输出契约**：进一步收紧 `gx` 的 stdout/stderr 规则，确保 markdown 文档输出可直接机读，说明性 CLI 文本统一走 `stderr`，`quiet` 模式不再把执行器/菜单说明泄漏到 `stdout`。
- **初始化参数校验**：`gx init project` 现在将 `--branch` 与 `--tag` 设为互斥，并把 human-oriented 初始化提示统一输出到 `stderr`。
- **Galaxy 环境初始化**：`gx init env` 在缺失时会一并创建默认的全局 `~/.galaxy/conf.toml`。

## [v0.12.4] - 2026-03-05

### 新增
- **自更新命令集**：新增 `gx self status/check/update/rollback/auto`。
- **自更新核心模块**：新增 `src/self_update/*`，覆盖策略/状态存储、manifest 下载、sha256 校验、安装回滚与健康检查。
- **仓库内更新清单**：新增 `updates/stable/manifest.json`、`updates/alpha/manifest.json`、`updates/beta/manifest.json` 及 `updates/README.md`。

### 变更
- **发布通道**：统一为 `stable|alpha|beta`（不再兼容 `pre`）。
- **默认清单源**：`manifest_base_url` 默认指向当前仓库 `updates/` 的 raw 地址。
- **版本更新流程**：`_gal/adm.gxl` 改为使用 `gx.patch_file` + marker 进行版本号 patch。

### 修复
- **更新语义**：`--dry-run` 不再依赖 `--yes`。
- **状态落盘**：补全 update 后半流程失败时的状态记录。
- **锁安全**：增加 stale lock 回收与 PID 存活检查。
- **回滚安全**：加强 `rollback --id` 校验，阻断路径穿越。
- **安装安全**：包内二进制查找要求唯一命中并忽略符号链接。
- **临时目录**：自更新临时目录改为自动清理。

## [0.10.1] - 2025-08-09

### 新增
- **Galaxy环境初始化**：新增Galaxy平台环境设置功能，支持自动化配置和Galaxy环境初始化
- **网络访问控制服务**：用新的网络访问控制服务替代旧的重定向服务，提供更好的安全性和访问管理
- **构件下载重定向支持**：增加构件下载的重定向功能，提高构件获取的可靠性和灵活性
- **移除gx.artifact功能**：移除旧版gx.artifact能力，推荐使用更灵活的下载替代方案（破坏性变更）
- **项目根目录检测**：新增`GXL_PRJ_ROOT`环境变量用于自动项目根目录发现

### 变更
- **依赖更新**：将`orion_variate`升级到v0.6.2（标签版本），带来增强的变量处理能力和比分支依赖更好的稳定性
- **工作流增强**：升级CI/CD工作流，改进构建和发布流程

### 安全性
- **访问控制增强**：实施网络访问控制服务替代传统重定向服务，改善安全态势和外部服务访问管理

### 修复
- **测试用例**：解决各种测试用例问题，提高测试可靠性
- **代码清理**：移除已弃用的artifact模块和相关解析器代码，降低复杂性和维护负担

## [0.10.0] - 2025-08-07

### 新增
- **增强模板引擎**：支持模板变量（gx.tpl），支持子目录和改进的渲染系统
- **TOML配置支持**：增强TOML文件读取功能，支持改进的解析器和验证（#5）
- **任务系统集成**：全面的任务执行框架，包括：
  - **任务调度和执行**：完整的任务生命周期管理，支持状态跟踪
  - **本地任务报告生成**：自动生成任务执行结果的YAML报告
  - **任务结果持久化**：结构化YAML输出，包含详细执行指标
  - **任务状态同步**：与中央任务管理服务实时同步
  - **试运行功能**：预执行验证，支持`@dryrun`注解和`--dry-run`CLI参数
  - **重试机制**：配置文件加载的自动重试，带有回退策略
  - **任务注解**：丰富的元数据支持（@task、@dryrun注解）
  - **HTTP回调集成**：与任务管理中心的集成
- **输出捕获系统**：标准输出和错误捕获，支持重定向功能
- **GXL环境变量**：支持GXL特定环境变量：
  - `GXL_PRJ_ROOT` 用于项目根目录检测
  - `GXL_CMD_ARG` 用于命令参数处理
  - `GXL_CMD_ARGS`（已弃用） -> `GXL_CMD_ARG`
- **事务支持**：通过`undo_hold`操作增加事务功能和回滚能力
- **WildMatch模式支持**：增加wildmatch库用于高级glob模式匹配
- **命令块语法**：增强命令执行的结构化块语法
- **异步执行引擎**：重构核心引擎使用async/await提高性能

### 变更
- **重构错误处理**：改进系统范围的错误报告，提供更好的错误消息和堆栈跟踪
- **重构流程结构**：现代化流程执行结构，增强异步支持
- **管道语法**：改进流程管道语法，支持更复杂操作
- **配置加载**：增强配置文件加载，支持重试机制和优雅处理

### 已弃用
- **传统重定向服务**：标记为移除，推荐使用网络访问控制服务
- **旧版构件服务**：gx.artifact功能已在v0.10.1版本中移除

### 修复
- **命令环境变量**：修复命令执行中的环境变量传播问题
- **任务配置**：正确处理缺失的任务配置文件或URL
- **日志重定向**：纠正任务执行中的标准输出/错误重定向问题
- **数据格式处理**：修复任务报告中的日期时间格式问题

### 开发者体验
- **增强CLI**：增加`--dry-run`命令行参数用于配置测试
- **更好调试**：当流程不存在时增加警告提示，改进错误消息
- **测试框架**：全面的测试套件添加，包括：
  - 任务系统测试用例
  - 配置加载测试
  - 输出捕获验证
  - 命令执行测试

## [0.9.2-beta.1] - 2024-09-05

之前的稳定版本。此更新日志涵盖从0.9.2-beta.1到0.10.1的变更。

---

**注意**：0.10.x版本相比0.9.x有显著的架构改进，特别是在任务管理、配置处理和系统集成方面。

### 迁移说明
- 环境变量名已标准化。将使用`GXL_CMD_ARG`替代已弃用的`GXL_CMD_ARGS`
- 任务配置文件应更新为新的YAML格式用于任务报告
- 使用新的任务系统时，可参考`examples/task-*.gxl`文件获取配置模式
- 重定向URL的请求头已从传统格式升级为新的访问控制格式
- 旧版本的`undo_flow`已被`undo_hold`替代用于事务操作
- 命令执行语法更新了块结构模式，需要更新现有`.gxl`文件语法

### 升级检查清单
- [ ] 更新环境变量声明从`GXL_CMD_ARGS`到`GXL_CMD_ARG`
- [ ] 检查所有配置文件是否使用YAML格式
- [ ] 验证任务报告URL配置是否适配新格式
- [ ] 测试旧的`.gxl`文件在新版本下的兼容性
- [ ] 确认事务回滚操作使用新的`undo_hold`语法
- [ ] 从gx.artifact迁移：将所有gx.artifact用法替换为gx.download，获得更好的灵活性和控制

---
### 相关Issue链接
- 任务系统设计: #5
- TOML支持: #5
- 输出捕获: #31, #32
- 重定向修复: #35
- CLI错误处理: #46

### 贡献者
特别感谢以下开发者在0.10.x版本周期中的贡献: @wukong, @sec-wukong, @tangxy1024, @tangxiangyan
