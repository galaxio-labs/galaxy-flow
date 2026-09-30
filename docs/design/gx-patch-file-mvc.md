# gx.patch_file 能力设计（MVC）

## 文档信息
- 文档名称: gx.patch_file 能力设计（MVC）
- 创建日期: 2026-03-04
- 版本: 1.0
- 状态: 已实现（MVP）

> 与实现的对应关系：`src/ability/patch/{model,view,controller}.rs`。
> 已核对：action 取值（`set` / `comment_line` / `uncomment_line` / `comment_block` /
> `uncomment_block`）与 `src/ability/patch/model.rs` 一致；参数与 marker 约定与
> `docs/gxl/inner/patch_file.md` 一致。

## 1. 目标与背景

### 1.1 背景
在工作流执行过程中，需要可控地修改配置文件内容，例如：
- 将 `version = 1.0` 改为 `version = v2.0`
- 将
  ```toml
  [features]
  res_depend_test = []
  ```
  批量注释为
  ```toml
  #[features]
  #res_depend_test = []
  ```

纯文本替换（直接按字符串匹配）有两个问题：
1. 目标不稳定，容易误改同名内容。
2. 缺乏可审计性，不清楚为什么改了这一行。

### 1.2 设计目标
- 通过**注释标记（marker）**精准定位可变更位置。
- 将变更操作统一为 `gx.patch_file`，支持值替换、行注释、块注释。
- 提供 `strict/dry_run/backup` 控制，保证安全和可回滚。

### 1.3 非目标
- 不做 AST 级语义改写（当前是行文本级 patch）。
- 不做自动冲突合并（如跨多人并发编辑）。

## 2. MVC 架构设计

### 2.1 Model（模型层）
文件: `src/ability/patch/model.rs`

核心模型：
- `PatchAction`
  - `Set`
  - `CommentLine`
  - `UncommentLine`
  - `CommentBlock`
  - `UncommentBlock`
- `GxPatchFile`（能力请求模型）
  - `file`: 目标文件路径
  - `marker`: 标记 ID
  - `action`: 动作
  - `value`: `set` 动作时的新值
  - `strict`: 严格模式（默认 true）
  - `dry_run`: 仅计算不落盘（默认 false）
  - `backup`: 写入前是否备份（默认 false）
  - `comment_prefix`: 注释前缀（默认 `#`）

### 2.2 Controller（控制层）
分为两段控制流程：

1. DSL Controller（解析控制）
   - 文件: `src/parser/inner/patch.rs`
   - 职责: 将 GXL 中 `gx.patch_file(...)` 参数解析并构建 `GxPatchFile`。

2. Runtime Controller（执行控制）
   - 文件: `src/ability/patch/controller.rs`
   - 职责:
     - 读取文件
     - 根据 `action + marker` 应用 patch
     - 执行 strict 校验
     - 按 `dry_run/backup` 决定是否写盘
     - 生成执行结果

### 2.3 View（视图层）
文件: `src/ability/patch/view.rs`

- `PatchView` 输出统一执行摘要：
  - action
  - file
  - marker
  - marker_hits
  - changed_lines
  - dry_run

该摘要写入 `Action.stdout`，用于任务记录与可观测性。

## 3. DSL 语法设计

### 3.1 指令
```gxl
gx.patch_file(
  file          : "...",
  action        : "set|comment_line|uncomment_line|comment_block|uncomment_block",
  marker        : "...",
  value         : "...",      # 仅 set 必填
  strict        : "true|false",
  dry_run       : "true|false",
  backup        : "true|false",
  comment_prefix: "#"
);
```

说明：当前 action 参数解析为字符串，布尔值也使用字符串（`"true"`/`"false"`）。

### 3.2 Marker 规范
- `set` 目标行标记: `@gxl:set(<id>)`
- `line` 目标行标记: `@gxl:line(<id>)`
- `block` 起止标记:
  - 开始: `@gxl:block(<id>)`
  - 结束: `@gxl:end(<id>)`
- marker 注释前缀支持 `#` 与 `//`，并支持紧贴写法（如 `#@gxl:set(id)`、`//@gxl:set(id)`）。

## 4. 行为规则

### 4.1 `set`
- 仅处理包含 `@gxl:set(id)` 的行。
- 在该行中定位赋值分隔符（`=` 或 `:`），替换 value 区段。
- 保留：
  - 原缩进
  - 键名和分隔符
  - marker 注释后缀

### 4.2 `comment_line` / `uncomment_line`
- 仅处理包含 `@gxl:line(id)` 的行。
- `comment_line`: 在缩进后插入 `comment_prefix`。
- `uncomment_line`: 在缩进后移除一个 `comment_prefix`（以及可选一个空格）。

### 4.3 `comment_block` / `uncomment_block`
- 在 `@gxl:block(id)` 与 `@gxl:end(id)` 之间处理内容行。
- 默认不改动 marker 行本身。
- 对中间行执行注释/反注释。

## 5. 安全策略

### 5.1 strict 模式
- 默认 `strict = true`。
- `set/line`：要求 marker 命中数必须为 1。
- `block`：要求有效 block 必须为 1 组（且开始在结束之前）。
- `block` 结构异常（例如嵌套 block、孤立 end、缺失 end）都会失败。
- 不满足时返回错误，停止执行。

### 5.2 dry_run
- 计算并统计变更，但不写回文件。

### 5.3 backup
- 当 `dry_run = false` 且存在实际改动时，先写 `目标文件.bak`。

### 5.4 comment_prefix 约束
- `comment_prefix` 不能为空字符串。
- 该约束在解析阶段和运行阶段都校验，避免误写盘。

## 6. 典型示例

### 6.1 版本号替换
源文件：
```toml
version = 1.0   # @gxl:set(version)
```

GXL：
```gxl
gx.patch_file(
  file   : "./Cargo.toml",
  action : "set",
  marker : "version",
  value  : "v2.0"
);
```

结果：
```toml
version = v2.0   # @gxl:set(version)
```

### 6.2 features 区块注释转换（添加/删除）
#### A. `comment_block`（添加注释）
源文件：
```toml
# @gxl:block(res_depend_test)
[features]
res_depend_test = []
# @gxl:end(res_depend_test)
```

GXL：
```gxl
gx.patch_file(
  file   : "./Cargo.toml",
  action : "comment_block",
  marker : "res_depend_test"
);
```

结果：
```toml
# @gxl:block(res_depend_test)
#[features]
#res_depend_test = []
# @gxl:end(res_depend_test)
```

#### B. `uncomment_block`（删除注释）
源文件：
```toml
# @gxl:block(res_depend_test)
#[features]
#res_depend_test = []
# @gxl:end(res_depend_test)
```

GXL：
```gxl
gx.patch_file(
  file   : "./Cargo.toml",
  action : "uncomment_block",
  marker : "res_depend_test"
);
```

结果：
```toml
# @gxl:block(res_depend_test)
[features]
res_depend_test = []
# @gxl:end(res_depend_test)
```

### 6.3 单行注释 / 反注释
源文件：
```toml
res_depend_test = []   # @gxl:line(res_depend_test)
```

注释 GXL：
```gxl
gx.patch_file(
  file   : "./Cargo.toml",
  action : "comment_line",
  marker : "res_depend_test"
);
```

反注释 GXL：
```gxl
gx.patch_file(
  file   : "./Cargo.toml",
  action : "uncomment_line",
  marker : "res_depend_test"
);
```

### 6.4 dry_run + backup + strict
```gxl
gx.patch_file(
  file   : "./Cargo.toml",
  action : "comment_block",
  marker : "res_depend_test",
  strict : "true",
  dry_run: "true",
  backup : "true"
);
```

说明：
- `dry_run = "true"` 时只统计变更，不写盘。
- `backup = "true"` 仅在实际写盘且有改动时生效（写 `*.bak`）。
- `strict = "true"` 下 marker 结构异常会直接失败。

### 6.5 自定义 comment_prefix
例如注释 Rust 行时使用 `//`：
```gxl
gx.patch_file(
  file          : "./src/main.rs",
  action        : "comment_line",
  marker        : "demo_marker",
  comment_prefix: "//"
);
```

### 6.6 set 动作（冒号分隔）
源文件：
```yaml
image: app:v1   # @gxl:set(image)
```

GXL：
```gxl
gx.patch_file(
  file   : "./deploy.yaml",
  action : "set",
  marker : "image",
  value  : "app:v2"
);
```

结果：
```yaml
image: app:v2   # @gxl:set(image)
```

### 6.7 常见失败写法（用于验错）
1. `set` 缺少 `value`（运行时报错）：
```gxl
gx.patch_file(
  file   : "./Cargo.toml",
  action : "set",
  marker : "version"
);
```

2. `comment_prefix` 为空字符串（解析报错）：
```gxl
gx.patch_file(
  file          : "./Cargo.toml",
  action        : "comment_line",
  marker        : "res_depend_test",
  comment_prefix: ""
);
```

## 7. 实现映射

- 能力注册入口: `src/ability/mod.rs`
- 能力分发接入: `src/model/components/gxl_block.rs`
- DSL 解析接入: `src/parser/inner/mod.rs`, `src/parser/stc_blk.rs`
- 语法解析: `src/parser/inner/patch.rs`
- 运行执行: `src/ability/patch/controller.rs`
- 模型定义: `src/ability/patch/model.rs`
- 结果视图: `src/ability/patch/view.rs`

## 8. 测试策略

### 8.1 单元测试
`src/ability/patch/controller.rs`
- `set` 替换正确性
- `comment_block`/`uncomment_block` 正反向
- strict 缺失 marker 失败

### 8.2 解析测试
`src/parser/inner/patch.rs`
- `set` 参数解析
- `comment_block` + 可选参数解析

### 8.3 集成建议（后续）
- 增加端到端 `.gxl` 流程用例，验证 `dry_run` 与 `backup`。

## 9. 后续扩展方向

- 多操作批处理（一次指令执行多个 patch op）。
- 支持 `expect_old`（旧值校验）防并发误改。
- 支持 JSON/TOML AST 模式（语义级替换）。
