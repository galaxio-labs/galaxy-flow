# 第二期实施指导文档

> **⚠️ 当前状态：本文档为历史设计与实施计划，其中描述的功能已下线**
>
> AI 能力目前在 galaxy-flow 中不可用：`orion-ai` 依赖在 `Cargo.toml` 中被注释；`gx.ai_fun` / `gx.ai_chat` 的解析与执行模块未编译（`src/parser/inner/mod.rs`、`src/ability/ai/mod.rs` 中相关 `pub mod` 已注释）；`ai_diagnose` 是 no-op。
>
> 因此文中涉及的 `examples/ai_fun_tools/` 等示例目录并未创建，相关命令在当前版本无法执行；旧的 `gflow` 二进制已统一为 `gx`。

## 概述

第二期实施目标是增强 AI 任务能力，实现工具调用和 Git 操作功能。本文档提供了详细的实施指导、技术要点和最佳实践。

## 目标回顾

### 核心目标
- **工具调用支持**：实现 AI 驱动的工具调用机制
- **Git 操作集成**：提供基础的 Git 命令执行能力
- **智能决策**：AI 根据任务描述自动选择工具
- **错误处理**：完善的错误处理和恢复机制

### 技术目标
- **函数注册表**：集成 orion_ai 的 FunctionRegistry
- **工具执行器**：实现 Git 工具的执行逻辑
- **参数解析**：扩展解析器支持工具调用参数
- **测试覆盖**：确保功能的稳定性和正确性

## 技术架构

### 系统流程图
```
用户输入 gx.ai_fun 指令
    ↓
解析器处理参数（tools, enable_function_calling）
    ↓
GxAIFun 构建函数注册表
    ↓
注册 GitFunctionExecutor 到注册表
    ↓
发送带函数定义的 AI 请求
    ↓
AI 返回函数调用决策
    ↓
执行 Git 工具调用
    ↓
返回执行结果和 AI 响应
```

### 核心组件关系
```
GxAIFun (用户接口)
    ↓
FunctionRegistry (函数管理)
    ↓
GitFunctionExecutor (工具执行)
    ↓
tokio::process::Command (实际 Git 操作)
```

## 详细实施步骤

### 步骤 1：增强 GxAIFun 结构

#### 1.1 添加新字段
在 `galaxy-flow/src/ability/ai_fun.rs` 中添加工具调用相关字段：

```rust
#[derive(Clone, Debug, Default, Getters, Setters, WithSetters, MutGetters)]
pub struct GxAIFun {
    // 现有字段...
    role: Option<String>,
    task: Option<String>,
    prompt: Option<String>,
    ai_config: Option<AiConfig>,

    // 新增字段
    tools: Option<Vec<String>>,         // 可用工具列表
    enable_function_calling: bool,      // 启用函数调用
}
```

#### 1.2 更新构造函数
```rust
impl GxAIFun {
    pub fn new() -> Self {
        Self {
            // 现有默认值...
            role: None,
            task: None,
            prompt: None,
            ai_config: None,

            // 新增默认值
            tools: None,
            enable_function_calling: false,
        }
    }

    // 添加工具设置方法
    pub fn with_tools(mut self, tools: Option<Vec<String>>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_enable_function_calling(mut self, enabled: bool) -> Self {
        self.enable_function_calling = enabled;
        self
    }
}
```

### 步骤 2：实现函数调用逻辑

#### 2.1 条件执行分支
修改 `execute_impl` 方法，添加函数调用支持：

```rust
async fn execute_impl(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
    ctx.append("gx.ai_fun");
    let mut action = Action::from("gx.ai_fun");
    let exp = EnvExpress::from_env_mix(vars_dict.global().clone());

    // 构建提示词
    let message = self.build_message(&exp)?;

    // 调用 AI 客户端
    let ai_config = self.get_ai_config(&vars_dict)?;
    let ai_client = AiClient::new(ai_config, None).err_conv()?;
    let role = self.get_role(&ai_client);

    if self.enable_function_calling {
        // 支持函数调用的执行
        self.execute_with_function_calling(&ai_client, &role, &message, vars_dict)
            .await?;
    } else {
        // 简单的 AI 对话
        self.execute_simple_chat(&ai_client, &role, &message)
            .await?;
    }

    action.finish();
    Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
}
```

#### 2.2 实现简单对话模式
```rust
async fn execute_simple_chat(
    &self,
    ai_client: &AiClient,
    role: &AiRoleID,
    message: &str,
) -> TaskResult {
    // 发送 AI 请求
    let ai_response = ai_client
        .smart_role_request(role, message)
        .await
        .err_conv()
        .with(format!("role:{}", role))?;

    // 处理 AI 响应
    self.log_ai_response(&ai_response);
    Ok(())
}
```

#### 2.3 实现函数调用模式
```rust
async fn execute_with_function_calling(
    &self,
    ai_client: &AiClient,
    role: &AiRoleID,
    message: &str,
    vars_dict: VarSpace,
) -> TaskResult {
    use orion_ai::function_calling::FunctionRegistry;
    use orion_ai::provider::AiRequest;

    // 创建函数注册表
    let mut registry = FunctionRegistry::new();

    // 注册可用的工具函数
    let functions = self.get_available_functions();
    for function in functions {
        registry.register_function(function.clone()).err_conv()?;
    }

    // 注册函数执行器
    let executor = std::sync::Arc::new(GitFunctionExecutor);
    for function_name in executor.supported_functions() {
        registry
            .register_executor(function_name, executor.clone())
            .err_conv()?;
    }

    // 构建 AI 请求
    let request = AiRequest::builder()
        .model(self.get_model(&vars_dict))
        .system_prompt(self.build_system_prompt())
        .user_prompt(message.to_string())
        .functions(functions)
        .enable_function_calling(true)
        .build();

    // 发送带函数调用的 AI 请求
    let ai_response = ai_client
        .send_request_with_functions(request, &registry)
        .await
        .err_conv()
        .with(format!("role:{}", role))?;

    // 处理 AI 响应
    self.log_ai_response(&ai_response);

    // 如果有函数调用，执行它们
    if let Some(tool_calls) = &ai_response.tool_calls {
        self.execute_tool_calls(tool_calls, &registry).await?;
    }

    Ok(())
}
```

### 步骤 3：实现 Git 工具执行器

#### 3.1 定义执行器结构
```rust
pub struct GitFunctionExecutor;

#[async_trait]
impl FunctionExecutor for GitFunctionExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "git-status" => self.execute_git_status(function_call).await,
            "git-add" => self.execute_git_add(function_call).await,
            "git-commit" => self.execute_git_commit(function_call).await,
            "git-push" => self.execute_git_push(function_call).await,
            "git-diff" => self.execute_git_diff(function_call).await,
            _ => Err(AiErrReason::from_logic("unknown function").to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![
            "git-status".to_string(),
            "git-add".to_string(),
            "git-commit".to_string(),
            "git-push".to_string(),
            "git-diff".to_string(),
        ]
    }
}
```

#### 3.2 实现 Git 操作方法
```rust
impl GitFunctionExecutor {
    async fn execute_git_status(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        let args = self.parse_arguments(&function_call.function.arguments)?;
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        match tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
            .await
        {
            Ok(output) => {
                let status = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(FunctionResult {
                    name: "git-status".to_string(),
                    result: json!({
                        "status": status,
                        "has_changes": !status.trim().is_empty()
                    }),
                    error: None,
                })
            }
            Err(e) => Ok(FunctionResult {
                name: "git-status".to_string(),
                result: serde_json::Value::Null,
                error: Some(format!("Failed to get git status: {}", e)),
            }),
        }
    }

    async fn execute_git_add(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        let args = self.parse_arguments(&function_call.function.arguments)?;
        let files = args
            .get("files")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AiErrReason::from_logic("files parameter required"))?;

        let file_list: Vec<String> = files
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();

        match tokio::process::Command::new("git")
            .args(["add"])
            .args(file_list)
            .output()
            .await
        {
            Ok(output) if output.status.success() => Ok(FunctionResult {
                name: "git-add".to_string(),
                result: json!({
                    "success": true,
                    "message": "Files added successfully"
                }),
                error: None,
            }),
            Ok(output) => {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                Ok(FunctionResult {
                    name: "git-add".to_string(),
                    result: serde_json::Value::Null,
                    error: Some(error_msg.to_string()),
                })
            }
            Err(e) => Ok(FunctionResult {
                name: "git-add".to_string(),
                result: serde_json::Value::Null,
                error: Some(format!("Failed to add files: {}", e)),
            }),
        }
    }

    // 类似实现其他 Git 操作...
}
```

### 步骤 4：扩展解析器支持

#### 4.1 更新解析器逻辑
```rust
// 在 galaxy-flow/src/parser/inner/ai_fun.rs 中
pub fn gal_ai_fun(input: &mut &str) -> Result<GxAIFun> {
    let mut ai_fun = GxAIFun::default();
    gal_keyword("gx.ai_fun", input)?;
    let props = action_call_args.parse_next(input)?;

    for one in props {
        let key = one.0.to_lowercase();
        match key {
            "role" => ai_fun.set_role(one.1.to_opt()),
            "task" => ai_fun.set_task(one.1.to_opt()),
            "prompt" => ai_fun.set_prompt(one.1.to_opt()),
            "ai_config" => ai_fun.set_ai_config(None),

            // 新增参数处理
            "tools" => ai_fun.set_tools(parse_tools_list(one.1)),
            "enable_function_calling" | "function_calling" => {
                ai_fun.set_enable_function_calling(
                    one.1.to_opt().map(|s| s.parse::<bool>().unwrap_or(false))
                );
            }
        }
    }
    Ok(ai_fun)
}

fn parse_tools_list(value: SecValueType) -> Option<Vec<String>> {
    value.to_str().map(|s| {
        s.split(',')
            .map(|tool| tool.trim().to_string())
            .collect::<Vec<_>>()
    })
}
```

#### 4.2 添加解析器测试
```rust
#[test]
fn ai_fun_with_tools() {
    let mut data = r#"
        gx.ai_fun( tools: "git status, git diff", enable_function_calling: true );"#;
    let obj = gal_ai_fun(&mut data).assert();
    assert_eq!(data, "");
    assert_eq!(
        obj.tools(),
        &Some(vec!["git status".to_string(), "git diff".to_string()])
    );
    assert_eq!(obj.enable_function_calling(), &true);
}

#[test]
fn ai_fun_single_tool() {
    let mut data = r#"
        gx.ai_fun( tools: "git status", function_calling: false );"#;
    let obj = gal_ai_fun(&mut data).assert();
    assert_eq!(data, "");
    assert_eq!(obj.tools(), &Some(vec!["git status".to_string()]));
    assert_eq!(obj.enable_function_calling(), &false);
}
```

### 步骤 5：创建测试用例

#### 5.1 单元测试
```rust
#[tokio::test]
async fn test_tools_parameter() {
    let ai_fun = GxAIFun::default()
        .with_tools(Some(vec!["git status".to_string(), "git diff".to_string()]))
        .with_enable_function_calling(true);

    assert_eq!(
        ai_fun.tools(),
        &Some(vec!["git status".to_string(), "git diff".to_string()])
    );
    assert_eq!(ai_fun.enable_function_calling(), &true);
}

#[tokio::test]
async fn test_function_definition_creation() {
    let ai_fun = GxAIFun::default();

    let git_status_func = ai_fun.create_git_status_function();
    assert_eq!(git_status_func.name, "git-status");
    assert_eq!(git_status_func.parameters.len(), 1);
    assert_eq!(git_status_func.parameters[0].name, "path");
}

#[tokio::test]
async fn test_available_functions() {
    let ai_fun = GxAIFun::default()
        .with_tools(Some(vec!["git status".to_string(), "git commit".to_string()]));

    let functions = ai_fun.get_available_functions();
    assert_eq!(functions.len(), 2);
    assert_eq!(functions[0].name, "git-status");
    assert_eq!(functions[1].name, "git-commit");
}
```

#### 5.2 集成测试
```rust
#[tokio::test]
async fn test_ai_with_git_tools() {
    // 跳过测试，如果没有 Git 仓库
    if !std::path::Path::new(".git").exists() {
        return;
    }

    let (context, mut def) = ability_env_init();
    def.global_mut()
        .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");

    let ai_fun = GxAIFun::default()
        .with_role(Some("developer".to_string()))
        .with_task(Some("检查 Git 状态".to_string()))
        .with_tools(Some(vec!["git status".to_string()]))
        .with_enable_function_calling(true);

    let result = ai_fun.async_exec(context, def).await;

    // 如果有 API 密钥，应该成功
    if load_key_dict("sec_deepseek_api_key").is_some() {
        result.assert();
    }
}
```

### 步骤 6：创建示例项目

#### 6.1 创建示例目录
```bash
mkdir -p galaxy-flow/examples/ai_fun_tools/_gal
```

#### 6.2 创建工作流文件
```gxl
// galaxy-flow/examples/ai_fun_tools/_gal/work.gxl
mod envs {
    env default {}
}

mod main {
    flow git_status_check {
        gx.ai_fun(
            role: "developer",
            task: "检查当前 Git 仓库状态",
            tools: ["git status"]
        );
    }

    flow smart_commit {
        gx.ai_fun(
            role: "developer",
            task: "生成并执行 Git 提交",
            tools: ["git status", "git add", "git commit"],
            enable_function_calling: true
        );
    }

    flow git_diff_analysis {
        gx.ai_fun(
            role: "developer",
            task: "查看代码变更并分析",
            tools: ["git diff"],
            enable_function_calling: true
        );
    }
}
```

#### 6.3 创建 README 文档
```markdown
# AI 工具调用功能示例

## 概述

本示例展示了第二期实现的 AI 工具调用功能，包括 Git 状态检查、智能提交和差异分析。

## 功能特性

### 1. Git 状态检查
```bash
gx run git_status_check
```

**预期输出**：
- AI 获取当前 Git 仓库状态
- 显示是否有未跟踪的文件
- 提供下一步操作建议

### 2. 智能提交
```bash
gx run smart_commit
```

**预期输出**：
- AI 检查仓库状态
- 自动生成提交消息
- 执行添加和提交操作

### 3. 差异分析
```bash
gx run git_diff_analysis
```

**预期输出**：
- AI 获取代码差异
- 分析变更内容
- 提供质量评估

## 使用要求

### 前提条件
1. **Git 仓库**：当前目录必须是 Git 仓库
2. **AI 服务**：需要配置 AI 服务密钥
3. **Galaxy-flow**：需要编译完成的项目

### 配置说明
```yaml
# ~/.galaxy/ai.yml
ai_config:
  model: "deepseek-chat"
  api_key: "your-api-key"
  function_calling: true
```

## 技术实现

### 核心组件
1. **GxAIFun** - 主要的 AI 任务执行器
2. **FunctionRegistry** - 函数注册和管理
3. **GitFunctionExecutor** - Git 工具执行器
4. **函数调用机制** - AI 驱动的工具选择

### 执行流程
1. 解析用户提供的工具列表
2. 注册对应的函数定义
3. AI 分析任务并选择工具
4. 执行工具调用并返回结果

## 常见问题

### Q: 如何添加新的工具？
A: 在 GitFunctionExecutor 中实现新的执行方法，并在 supported_functions 中注册。

### Q: 工具调用失败怎么办？
A: 系统会提供详细的错误信息，包括失败原因和可能的解决方案。

### Q: 如何调试工具调用？
A: 使用 `gx run -d 1 <flow>` 启用调试日志，查看详细的执行过程。
```

## 技术要点

### 1. 错误处理策略
- **网络错误**：自动重试，最多 3 次
- **工具错误**：返回详细的错误信息和建议
- **AI 服务错误**：优雅降级到基础对话模式

### 2. 性能优化
- **异步执行**：所有 Git 操作都是异步的
- **并发限制**：支持最大并发任务数配置
- **结果缓存**：相同参数的调用结果会被缓存

### 3. 安全考虑
- **参数验证**：所有工具参数都经过验证
- **路径安全**：限制 Git 操作在指定目录
- **权限控制**：检查执行权限

## 测试策略

### 测试类型
1. **单元测试**：测试各个组件的独立功能
2. **集成测试**：测试组件间的协作
3. **端到端测试**：测试完整的工作流程

### 测试覆盖
- Git 状态检查 (git_status)
- 文件添加操作 (git_add)
- 提交操作 (git_commit)
- 推送操作 (git_push)
- 差异查看 (git_diff)

## 验收标准

### 功能验收
- [ ] 工具调用功能正常工作
- [ ] Git 操作能够正确执行
- [ ] AI 能够智能选择工具
- [ ] 错误处理机制完善

### 质量验收
- [ ] 所有测试通过
- [ ] 代码覆盖率达到 80% 以上
- [ ] 文档完整且准确
- [ ] 示例可以正常运行

### 性能验收
- [ ] 工具调用响应时间 < 10秒
- [ ] 内存使用合理
- [ ] 并发执行稳定

## 总结

第二期实施将显著增强 AI 任务能力，为用户提供智能化的工具调用功能。通过完善的 Git 操作集成，用户可以实现自动化的代码管理工作流。

**预计完成时间**：1-2 天
**当前进度**：60%
**下一步**：完成剩余的实现和测试
