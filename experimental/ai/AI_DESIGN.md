
# GXL AI-Native 模块架构设计
# Rust实现版 - Version 1.0

## 设计目标
将GXL AI-Native设计从Shell/脚本实现迁移到原生Rust实现，实现更强大、类型安全的AI工作流引擎。

## 核心架构组件

### 1. AI抽象层（Abstract layer）
位于 `src/ai/` 目录
- `AiProvider`: AI提供商trait定义
- `AiRequest`: 统一请求结构
- `AiResponse`: 统一响应结构
- `AiCapability`: AI能力枚举（analyze/suggest/generate/refactor等）

### 2. 提供商实现（Providers）
位于 `src/ai/providers/` 目录
- `OpenAiProvider`: GPT系列API封装
- `AnthropicProvider`: Claude系列API封装
- `OllamaProvider`: 本地模型支持
- `MockProvider`: 测试用mock实现

### 3. 配置系统（Configuration）
位于 `src/ai/config/` 目录
- `AiConfig`: 全局配置管理
- `ModelRouting`: 智能路由配置
- `TokenManager`: token使用追踪和管理
- `EnvironmentLoader`: 环境变量管理

### 4. 上下文系统（Context System）
位于 `src/ai/context/` 目录
- `WorkspaceContext`: 工作空间上下文
- `GitContext`: Git仓库上下文
- `FileContext`: 文件上下文
- `DiffContext`: 差异上下文

### 5. GXL语法集成（Grammar Integration）
位于 `src/parser/ai_grammar.rs`
- Ai语法扩展解析器
- 工作流表达式求值
- 上下文绑定和解析

### 6. Git集成（Git Integration）
位于 `src/git_ai/` 目录
- Git状态监控
- 智能提交生成
- 代码审核
- Changelog自动生成

### 7. CLI接口（CLI Interface）
位于 `src/cmd/ai/` 目录
- AI工作流命令行
- 交互式确认
- 输出格式化
- 错误处理

## 技术栈选择

1. **异步运行时**: tokio
2. **HTTP客户端**: reqwest with json
3. **配置管理**: serde_json + serde_yaml
4. **类型安全**: 全项目类型化，避免任何字符串拼接
5. **错误处理**: thiserror 自定义错误类型
6. **日志**: tracing (兼容log)
7. **测试**: mockall + tempfile

## 零配置启动路径

用户只需：
1. 设置环境变量: `OPENAI_API_KEY`, `CLAUDE_API_KEY`
2. 运行: `gx ai-smart-commit`
3. 无需任何额外配置（嵌入式默认配置）

## 安全设计

1. API密钥仅从环境变量读取，绝不存储磁盘
2. 敏感信息自动过滤（基于regex匹配）
3. Token使用追踪（防止攻击）
4. 本地模型优先降低网络风险

## 性能目标

1. 第一次调用<500ms（缓存配置）
2. 批量处理支持流式输出
3. 大数据文件分块分析
4. 并行多个AI提供商调用

## 测试策略

1. 单元测试: core, config, providers
2. 集成测试: git交互, 上下文构建
3. 端到端: 完整工作流
4. Mock测试: 无网络依赖测试

## 文件结构

```
galaxy-flow/
├── src/
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── provider.rs
│   │   ├── request.rs
│   │   ├── response.rs
│   │   ├── capabilities.rs
│   │   ├── providers/
│   │   │   ├── openai.rs
│   │   │   └── mock.rs
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── model.rs
│   │   │   └── token.rs
│   │   ├── context/
│   │   │   ├── mod.rs
│   │   │   ├── workspace.rs
│   │   │   └── git.rs
│   │   └── error.rs
│   ├── git_ai/
│   │   ├── mod.rs
│   │   ├── smart_commit.rs
│   │   ├── code_review.rs
│   │   └── changelog.rs
│   └── cmd/
│       └── ai/
│           ├── mod.rs
│           └── commands.rs
```

## 集成路线图

阶段1: 基础AI框架 (1-2天)
- 核心抽象层
- OpenAI Provider实现
- 配置系统

阶段2: Git集成 (2-3天)
- Git状态监控
- 智能提交
- 上下文收集

阶段3: 高级能力 (3-4天)
- Claude Provider
- Ollama本地模型
- Token管理

阶段4: 工作流引擎 (2-3天)
- GXL语法扩展
- CLI命令
- 端到端测试

阶段5: 生产增强 (持续)
- 性能优化
- 错误处理
- 用户体验

阶段2: Git集成 (Week
