# Changelog

All notable changes to the Galaxy Flow project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.13.15] - 2026-09-18

### Added
- **GXL docs mirror tooling**: Added `scripts/sync-gxl-docs.sh` (`sync` / `check` / `list`) to regenerate and verify the `operator-docs` copy of `docs/gxl/`. The mirror boundary is explicit: `docs/gxl/example/*.md` is owned by `operator-docs` and is never overwritten.

### Documentation
- **`gx init project` flags corrected**: README and the guide now document the real options (`--repo` / `--path` / `--branch` / `--tag`) instead of the non-existent `--tpl`, and state that init runs locally unless `--repo`/`--path` is given (`--branch` and `--tag` are mutually exclusive).
- **README AI status**: `--ai` is documented as currently ineffective (`ai_diagnose` is a no-op), and the `gx run` option list now includes the previously missing `--log` and `-q/--quiet`.
- **Single GXL syntax reference**: removed the stale duplicate `docs/syntax.md`; `docs/gxl/syntax.md` (also what `gx doc gxl` renders) is now the only syntax doc, with an added annotation section and `gx.sn` added to the built-in ability list.
- **Structure and design docs realigned**: merged the duplicated `docs/structure/*-actual.md` into `docs/structure/*.md`, and corrected claims that no longer matched the code — the `self_update` file list, the removed init template directory, `wp-self-update = "0.3"`, and the release manifest repository name.
- **Contributor guide rewritten**: `AGENTS.md` still described a two-binary (`gflow` / `gprj`) layout that no longer exists. It now documents the single `gx` binary, the real build/test/clippy commands (including `-D warnings`), and the fact that `docs/` Markdown is compiled into the binary via `include_str!`.

### Changed
- **Release manifest repository unified**: `.github/workflows/release.yml` now checks out and pushes to `galaxio-labs/get`, completing the earlier `galaxy-sec` → `galaxio-labs` migration.

### Fixed
- **Parser robustness**: `extern_parse` handled `DslStatus::Data` with `todo!()`; it now returns an error instead of panicking.
- **Clippy on newer toolchains**: removed redundant borrows in `debug!` / `info!` / `format!` arguments that `clippy::useless_borrows_in_formatting` rejects on rustc 1.98+ (CI runs clippy with `-D warnings`).

### Removed
- **Repo-local installer**: Removed `install.sh`'s install logic and the `updates/{stable,alpha,beta}/manifest.json` manifests it read. That chain was unmaintained (stable pinned at `0.12.4` with placeholder all-zero checksums, so installs silently skipped verification) and it read a different manifest source than `gx self`. `install.sh` is now a deprecation stub that exits with a pointer to the official installer (`https://get.warpparse.ai/inst-x.sh`); `gx self` still reads `galaxio-labs/get`.
- **Superseded init templates**: removed `app/gx/init/_gal/`. `gx init project` has generated its scaffold from `src/templates/` (embedded via `include_str!`) since the init templates moved there, and nothing referenced the old directory.
- **Uncompiled AI code**: the AI implementation commented out of the build (`src/ability/ai/*`, `src/parser/inner/ai_*.rs`, `src/AI_DESIGN.md`) moved to `experimental/ai/` with a restore guide. `galaxy_flow::ability::ai` is no longer part of the library API.

## [v0.13.14] - 2026-05-04

### Added
- **GXL serial number ability**: Added `gx.sn` for reading and updating simple numeric serial number files. The default action reads the current number and exports it as `SN` without writing the file; `action: "add"` increments and writes the next number; `action: "reset"` writes and exports `1`.
- **Documentation topic**: Added `gx doc gx.sn` and included `gx.sn` in the built-in documentation topic list and GXL inner ability index.

## [v0.13.13] - 2026-05-04

### Changed
- **Dependency upgrades**: Bumped orion-error to 0.8, orion_conf to 0.7, orion-sec to 0.6, orion-infra to 0.7, orion-variate to 0.13, and orion-accessor to 0.8.
- **Structured error model**: Migrated `RunReason`, `ExecReason`, and `GxlReason` business variants to unit enum variants, keeping dynamic diagnostic text on `StructError` detail/source/context instead of enum payloads.
- **orion-error v0.8 source handling**: Replaced selected `map_err` conversions at IO, HTTP, template walking, JSON serialization, and UTF-8 decoding boundaries with `source_err` or `source_raw_err` so underlying sources are preserved for reports.

### Removed
- **Legacy error compatibility**: Continued removing `.owe()`-era compatibility paths in favor of the current orion-error 0.8 APIs.

## [v0.13.12] - 2026-04-29

### Changed
- **Dependency upgrades**: Bumped orion-error to 0.7, orion_conf to 0.6, orion-sec to 0.5, orion-infra to 0.6, orion-variate to 0.12, and orion-accessor to 0.7.

- **orion-error v0.7 API migration**: Refactored error-handling calls across the entire codebase to match the new v0.7 API surface:
  - `WithContext::want()` renamed to `WithContext::doing()`, and `.with()` renamed to `.with_context()` for contextual error chaining.
  - Trait imports reorganized: `ErrorOwe`, `ErrorOweBase` moved to `orion_error::compat_traits`; `ToStructError`, `ContextRecord` moved to `orion_error::traits_ext`.
  - `OperationContext::want()` → `OperationContext::doing()` and `.with()` → `.with_context()` in ability modules (tpl, archive, load, version).
  - Added `DomainReason` trait implementations for `RunReason` and `GxlReason` custom error enums.
  - Test utilities migrated: `TestAssertWithMsg` → `orion_error::testcase::TestAssertWithMsg`, and added `TestAssert` usage in unit tests.
  - Replaced all deprecated `ErrorOwe` usage with `ErrorOweBase` + `UvsReason` across the codebase, removing 80+ deprecation warnings entirely.
  - Replaced category-specific `.owe_*()` shortcut methods with `.owe(UvsReason::*.into())` for explicit error classification.

## [v0.13.11] - 2026-04-21

### Added
- **Streaming shell output**: Added `stream: "true"` support to the shared shell option model used by `gx.cmd`, `gx.shell`, and `gx.read_cmd`, allowing long-running commands to forward stdout/stderr to the current session in real time while still preserving captured output and exit codes in command results.

### Documentation
- **GXL shell docs refresh**: Updated `docs/gxl` command, shell, read, syntax, help, and example pages to document the new streaming mode, including ansible-oriented examples and notes about stdout/stderr interleaving during live output.

### Fixed
- **Long-running command visibility**: `gx.cmd` no longer appears stalled during quiet long-running operations that do produce output; streamed execution now surfaces output immediately instead of waiting for the command to finish before printing buffered content.

## [v0.13.10] - 2026-04-07

### Changed
- **Self-update channel defaults**: `gx self check` and `gx self update` now default `--channel` to `stable`, matching the main-branch release channel without requiring the flag to be passed every time.
- **Self-check human output**: Reworked `gx self check` terminal output into a status report that highlights channel, current version, remote version, and semver relation, and suggests the update command when a newer version is available.

### Fixed
- **Remote project subdir init source**: Corrected `gx init project --path <subdir>` remote initialization so it copies the selected remote subdirectory contents into `./_gal/` instead of resolving against an intermediate cached checkout and ending up with mismatched template contents.
- **Init failure cleanup**: When remote project initialization fails after creating `./_gal/`, the command now removes the whole temporary target directory instead of only trying to delete an empty folder.

## [v0.13.9] - 2026-04-07

### Fixed
- **Init project subdir flattening**: `gx init project --path <subdir>` now places the selected template contents directly under `./_gal/` instead of leaving an extra nested directory such as `./_gal/rust/`.
- **Init target safety guard**: Added an internal boundary check so project init cleanup only reorganizes paths that were downloaded inside `./_gal/`, preventing accidental moves or deletions outside the init target.
- **CLI and shell test isolation**: Hardened loader, command, shell, and activity tests against shared working-directory pollution by pinning them to the workspace root and using stable absolute script/config paths where needed.
- **Module update config test isolation**: Reworked the `gx mod update` user-config regression test to run in a temporary project workspace so it no longer hits repository-local git extern modules or stalls on remote update paths.

## [v0.13.8] - 2026-04-06

### Changed
- **Repository ownership alignment**: Updated project, release, manifest, and installer endpoints from the legacy `galaxy-sec` organization to `galaxio-labs`.
- **Init project defaults**: Updated `gx init project` default template repository and related CLI help text to use `https://github.com/galaxio-labs/prj-tpl.git`.
- **Self-update sources**: Switched self-update manifest and install script defaults to the new `galaxio-labs` repository paths.

### Documentation
- **README and guides**: Refreshed README, CLI guide, syntax notes, self-update design docs, and updates documentation to match the current repository ownership and release endpoints.
- **Release metadata**: Synchronized version metadata and manifest template references for `0.13.8`.

### Fixed
- **Examples and tests**: Updated embedded example, template, and test repository URLs so init flows, parser examples, and template loading no longer point at retired `galaxy-sec` paths.

## [v0.13.5] - 2026-03-23

### Changed
- **Init project CLI redesign**: Replaced `--tpl` with separate `--repo` and `--path` options.
  - `gx init project` → local init (offline, creates basic work.gxl and adm.gxl)
  - `gx init project --path rust` → remote init from default repo's subdir
  - `gx init project --repo <url>` → remote init from custom repo
- **Default repo**: `https://github.com/galaxio-labs/prj-tpl.git` is used when `--path` is specified without `--repo`.
- **Parameter validation**: `--branch` and `--tag` now require `--repo` or `--path`.

### Fixed
- **Directory cleanup**: If copy to `_gal` fails, the empty directory is now cleaned up.

## [v0.13.4] - 2026-03-15

### Changed
- **Command success semantics**: Renamed the shell option field used by `gx.cmd`, `gx.shell`, and `gx.read_cmd` from `expect` to `ok_codes`, making it explicit that it represents the allowlist of successful exit codes.
- **Action result model**: Switched command action results to structured `exit_code`, `stdout`, and `stderr` fields instead of merging stdout/stderr into a single output field.
- **Module update result reporting**: `gx mod update` now summarizes git extern modules per config file and emits a final project-level update summary.

### Fixed
- **Module update config collection**: When both `./_gal/work.gxl` and `./_gal/adm.gxl` exist, `gx mod update` now keeps both inputs instead of dropping an existing config.
- **CLI output contract**: Human-oriented `gx mod update` result summaries now go to `stderr`, keeping `stdout` free of explanatory text.
- **Parser and regression coverage**: Added `ok_codes` parsing coverage and execution assertions for `exit_code/stdout/stderr` command results.

## [v0.13.0] - 2026-03-10

### Added
- **Unified `gx` CLI**: Added `gx` as the sole shipped command entrypoint, covering `run`, `adm`, `init`, `mod`, `doc`, `check`, and `self`.
- **Compact aliases**: Added `grun` -> `gx run` and `gadm` -> `gx adm` alias-based entry support.
- **CLI doc renderer**: Added `gx doc` topic indexing and terminal-friendly markdown rendering with `--markdown` raw output support.

### Changed
- **Project init assets**: Moved local init templates from `app/gprj/init` to `app/gx/init`.
- **Release packaging**: Release workflow, installer, and self-update now package and verify only the `gx` binary.
- **Workspace docs**: Updated README and guide docs to present `gx` as the default and only binary entrypoint.
- **CLI command model**: Renamed module management to `gx mod update`, split `gx run` and `gx adm` into dedicated clap wrappers with independent help text, and clarified `project`-oriented wording across the CLI.
- **CLI runtime setup**: Unified `gx init project` with the same runtime initialization path used by other commands, including log setup and `~/.galaxy/conf.toml` loading.

### Removed
- **Legacy binaries**: Removed shipped `gflow` and `gprj` binaries from the workspace and release artifacts.
- **Legacy CLI docs**: Removed dedicated `gflow` and `gprj` usage pages from the guide.
- **Legacy gx compatibility surface**: Removed deprecated `gx conf`, `gx init prj`, `gx init prj-with-local`, `gx update mod`, `--mod_up`, `--conf-work`, `--conf-adm`, and other obsolete compatibility flags.

### Fixed
- **Local workspace bootstrap**: Changed the repository `_gal/work.gxl` extern path to use `./_gal/` instead of `${GXL_START_ROOT}`, avoiding parse failures when `GXL_START_ROOT` is unavailable.
- **Module update semantics**: `gx mod update` now loads `~/.galaxy/conf.toml` consistently and fails fast when neither `./_gal/work.gxl` nor `./_gal/adm.gxl` exists.
- **CLI exit behavior**: `gx adm` now returns non-zero when all flows fail, while `gx run` / `gx adm` without an explicit flow keep the menu behavior and return success.
- **Output contract**: Tightened `gx` stdout/stderr behavior so markdown doc output stays machine-clean, explanatory CLI messages move to `stderr`, and quiet mode suppresses executor/menu chatter instead of leaking it to `stdout`.
- **Init command validation**: `gx init project` now treats `--branch` and `--tag` as mutually exclusive and routes human-oriented init messages to `stderr`.
- **Galaxy environment bootstrap**: `gx init env` now also creates the default global `~/.galaxy/conf.toml` when missing.

## [v0.12.4] - 2026-03-05

### Added
- **Self-update command set**: Added `gx self status/check/update/rollback/auto` for upgrade management.
- **Self-update runtime module**: Added `src/self_update/*` covering policy/state storage, manifest download, checksum verification, install/rollback, and health check.
- **Repository manifests**: Added `updates/stable/manifest.json`, `updates/alpha/manifest.json`, `updates/beta/manifest.json`, and `updates/README.md`.

### Changed
- **Release channels**: Unified channels to `stable|alpha|beta` (removed `pre` compatibility mode).
- **Default manifest source**: `manifest_base_url` now points to this repository raw path under `updates/`.
- **Version automation**: Switched project version bump flow to `gx.patch_file` marker-based patching in `_gal/adm.gxl`.

### Fixed
- **Update semantics**: `--dry-run` no longer requires `--yes`.
- **State persistence**: Expanded failure-state recording for post-manifest update stages.
- **Lock safety**: Added stale-lock recovery with PID liveness checks.
- **Rollback robustness**: Hardened backup id validation and prevented path traversal via `rollback --id`.
- **Installer safety**: Enforced unique binary detection and ignored symlink hits in package scanning.
- **Temp artifacts**: Added automatic cleanup of self-update temporary directories.

## [v0.12.0] - 2026-02-25

### Changed
- **依赖升级**: 核心依赖迁移至 crates.io 注册版本
  - `orion_conf`: `~0.1` → `0.4` (trait 重命名: `Yamlable`→`YamlIO`, `Tomlable`→`TomlIO`, `JsonAble`→`JsonIO`, `IniAble`→`IniIO`; 方法重命名: `from_yml`→`load_yaml`, `save_yml`→`save_yaml` 等)
  - `orion-sec`: `v0.2.0` (git) → `0.3` (registry)
  - `orion-infra`: `v0.3.1` (git) → `0.4` (registry)
  - `orion-variate`: `v0.9.1` (git) → `0.10` (registry)
- **模块迁移**: `orion-variate` 中的 `addr`、`types`、`update`、`archive` 模块迁移至新增依赖 `orion-accessor = 0.5.4`

### Removed
- **临时移除 orion-ai**: 因 `orion-ai` v0.2.1 依赖旧版 `orion-variate`/`orion-sec` 产生版本冲突，暂时移除 AI 相关功能，待 `orion-ai` 升级后恢复
  - 移除 `gx.ai_chat`、`gx.ai_task`、`gx.ai_regist` 解析与执行
  - `ai_diagnose` 暂时禁用
  - Galaxy 环境初始化不再生成 AI 配置文件

## [v0.10.0-alpha.1] - 2024-08-07
### ✨ 新增功能

#### 1. 核心功能增强
- **RedirectService 集成**：重定向服务系统

#### 2. 环境初始化增强 (Galaxy::env_init)
- 新增 `~/.galaxy/redirect.yml` 自动生成功能

#### 3. 命令行增强
- `gprj init env` 支持完整的RedirectService环境配置
- 模板系统优化，支持分支和标签筛选

## 📞 支持
如有问题，请运行：
```bash
gm init env --debug 3
```
## [0.10.1] - 2025-08-09

### Added
- **Galaxy Environment Initialization**: Added environment setup functionality for Galaxy platform, enabling automated configuration and initialization of Galaxy environments.
- **Network Access Control Service**: Replaced the legacy redirect service with a new network access control service, providing better security and access management.
- **Artifact Download Redirection Support**: Added redirect capabilities for artifact downloading, improving reliability and flexibility in artifact retrieval.
- **Removed gx.artifact Ability**: Removed the legacy gx.artifact capability in favor of more flexible download alternatives (breaking change).
- **Project Root Detection**: Added GXL_PRJ_ROOT environment variable for automatic project root discovery.

### Changed
- **Dependency Updates**: Upgraded `orion_variate` from v0.6.2 (tagged release), bringing enhanced variable handling capabilities and improved stability over branch-based dependencies.
- **Workflow Enhancements**: Updated CI/CD workflows with improved build and release processes.

### Security
- **Access Control Enhancement**: Implemented network access control service replacing the legacy redirect service, improving security posture and access management for external services.

### Fixed
- **Test Cases**: Resolved various test case issues and improved test reliability.
- **Code Cleanup**: Removed deprecated artifact module and related parser code, reducing complexity and maintenance burden.

## [0.10.0] - 2025-08-07

### Added
- **Enhanced Template Engine**: Added support for template variables (gx.tpl) with subdirectory support and improved render system.
- **TOML Configuration Support**: Enhanced TOML file reading capabilities with improved parser and validation (#5).
- **Task System Integration**: Comprehensive task execution framework with:
  - **Task Scheduling and Execution**: Full task lifecycle management with state tracking
  - **Local Task Report Generation**: Automatically generate YAML reports for task execution results
  - **Task Result Persistence**: Structured YAML output with detailed execution metrics
  - **Task Status Synchronization**: Real-time sync with central task management services
  - **Dry Run Functionality**: Pre-execution validation with `@dryrun` annotations and `--dry-run` CLI flag
  - **Retry Mechanisms**: Automatic retry for configuration file loading with backoff strategies
  - **Task Annotations**: Rich metadata support (@task, @dryrun annotations)
  - **HTTP Callback Integration**: Seamless integration with task management centers
- **Output Capture System**: Standard output and error capture with redirection capabilities
- **GXL Environment Variables**: Support for GXL-specific environment variables:
  - `GXL_PRJ_ROOT` for project root detection
  - `GXL_CMD_ARG` for command argument handling
  - `GXL_CMD_ARGS` (deprecated) -> `GXL_CMD_ARG`
- **Transaction Support**: Added transaction capabilities with rollback functionality through `undo_hold` operations
- **WildMatch Pattern Support**: Added wildmatch crate for advanced glob pattern matching
- **Command Block Syntax**: Enhanced command execution with structured block syntax
- **Async Execution Engine**: Refactored core engine to use async/await for better performance

### Changed
- **Refactored Error Handling**: Improved error reporting across the system with better error messages and stack traces
- **Refactored Flow Structure**: Modernized flow execution structure with enhanced async support
- **Pipeline Syntax**: Improved flow pipe syntax supporting more complex operations
- **Configuration Loading**: Enhanced configuration file loading with retry mechanisms and graceful handling

### Deprecated
- **Legacy Redirect Service**: Marked for removal in favor of network access control service
- **Legacy Artifact Service**: The gx.artifact ability has been removed in v0.10.1

### Fixed
- **Command Environment Variables**: Fixed issues with environment variable propagation in command execution
- **Task Configuration**: Proper handling when task configuration files or URLs are missing
- **Log Redirection**: Corrected issues with standard output/err redirection during task execution
- **Data Format Handling**: Fixed datetime format issues in task reporting

### Developer Experience
- **Enhanced CLI**: Added `--dry-run` command line parameter for testing configurations
- **Better Debugging**: Added warning prompts when flows don't exist and improved error messages
- **Testing Framework**: Comprehensive test suite additions including:
  - Task system test cases
  - Configuration loading tests
  - Output capture verification
  - Command execution tests

## [0.9.2-beta.1] - 2024-09-05

Previous stable release. This changelog covers changes from 0.9.2-beta.1 to 0.10.1.

---
**Note**: Version 0.10.x represents a significant evolution from 0.9.x with major architectural improvements, especially in task management, configuration handling, and system integration.

### Migration Notes
- **Environment variable names have been standardized. Update scripts using `GXL_CMD_ARGS` to `GXL_CMD_ARG`
- **Task configuration files should be updated to use new YAML format for task reports
- **For using the new task system, refer to `examples/task-*.gxl` files for configuration patterns
- **Redirect URL request headers have been upgraded from legacy format to new access control format
- **Legacy `undo_flow` has been replaced with `undo_hold` for transaction operations
- **Command execution syntax updated with new block structure pattern, requiring updates to existing `.gxl` file syntax
- **Migrate from gx.artifact**: Replace any gx.artifact usage with gx.download for better flexibility and control

### Upgrade Checklist
- [ ] Update environment variable declarations from `GXL_CMD_ARGS` to `GXL_CMD_ARG`
- [ ] Verify all configuration files use YAML format
- [ ] Validate task report URL configurations for new format compatibility
- [ ] Test compatibility of existing `.gxl` files with new version
- [ ] Confirm transaction rollback operations use new `undo_hold` syntax

---
### Related Issue Links
- Task System Design: #5
- TOML Support: #5
- Output Capture: #31, #32
- Redirect Fixes: #35
- CLI Error Handling: #46
- Download Improvements: #71
- Refactor Redirect Service: #66

### Contributors
Special thanks to the following developers for their contributions during the 0.10.x release cycle: @wukong, @sec-wukong, @tangxy1024, @tangxiangyan, @可乐加冰
