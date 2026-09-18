# AI Fun 基础功能示例

> **⚠️ 当前状态：功能已下线，本文档仅作历史留存**
>
> AI 能力目前在 galaxy-flow 中不可用：`orion-ai` 依赖在 `Cargo.toml` 中被注释；`gx.ai_fun` / `gx.ai_chat` 的解析与执行模块未编译（`src/parser/inner/mod.rs`、`src/ability/ai/mod.rs` 中相关 `pub mod` 已注释）。本示例当前执行会报 `call not found: gx.ai_fun`（该能力未接入）。
>
> 下面的命令用的是旧 `gflow` 二进制，该二进制已不存在（现已统一为 `gx`）；即使换成 `gx`，也需要等 `gx.ai_fun` 重新接入后才能运行。

## 概述

本示例展示了 Galaxy-flow 第一期实现的 `gx.ai_fun` 基础功能。这是一个全新的 AI 任务执行框架，允许用户在 gxl 工作流中直接调用 AI 功能。

## 功能特性

### 基础 AI 对话
- 支持与 AI 进行自然语言对话
- 可以指定 AI 角色（如 "developer"）
- 支持自定义提示词
- 集成现有的 orion_ai 框架

### 语法支持
```gxl
mod main {
    flow chat_test {
        gx.ai_fun(
            role: "developer",
            prompt: "请回答：1+1=?"
        );
    }
}
```

## 使用方法

### 前提条件
1. 配置 AI 服务密钥（如 DeepSeek API）
2. 确保 galaxy-flow 已正确编译
3. 具备网络连接访问 AI 服务

### 基本用法

#### 1. 简单对话
```bash
# 历史命令（旧 gflow 二进制，当前不可用）
# ./target/debug/gflow -f examples/ai_fun_basic/_gal/work.gxl chat_test
#
# 当前 CLI 形式（需等 gx.ai_fun 恢复接入后才能运行）
# cd examples/ai_fun_basic && gx run chat_test
```

#### 2. 任务描述
```bash
# 历史命令（旧 gflow 二进制，当前不可用）
# ./target/debug/gflow -f examples/ai_fun_basic/_gal/work.gxl task_test
```

#### 3. 组合参数
```bash
# 历史命令（旧 gflow 二进制，当前不可用）
# ./target/debug/gflow -f examples/ai_fun_basic/_gal/work.gxl combined_test
```

## 参数说明

### 支持的参数
- `role`: AI 角色名称（可选，默认为 "developer"）
- `task`: 任务描述（可选）
- `prompt`: 提示词（可选）
- `ai_config`: AI 配置（可选，暂未实现）

### 参数优先级
1. 如果同时提供 `task` 和 `prompt`，两者会合并
2. 如果两者都未提供，使用默认提示词："请回答我的问题。"

## 示例展示

### 示例 1：基础数学问题
```gxl
flow chat_test {
    gx.ai_fun(
        role: "developer",
        prompt: "请回答：1+1=?"
    );
}
```

**预期输出：**
```
AI Response:
Content: [角色: ai-role: developer]

1+1=2
Model: deepseek
Timestamp: 2025-08-25T13:15:55.368468+08:00
```

### 示例 2：概念解释
```gxl
flow task_test {
    gx.ai_fun(
        role: "developer",
        task: "请解释什么是人工智能"
    );
}
```

**预期输出：**
```
AI Response:
Content: [角色: ai-role: developer]

人工智能（Artificial Intelligence，简称 AI）是计算机科学的一个分支，旨在开发能够模拟人类智能的系统或机器...
[详细解释内容]
```

### 示例 3：组合任务
```gxl
flow combined_test {
    gx.ai_fun(
        role: "developer",
        task: "分析当前时间的重要性",
        prompt: "请考虑时间在软件开发中的作用"
    );
}
```

## 技术实现

### 核心组件
1. **GxAIFun** - 主要的 AI 任务执行器
2. **解析器支持** - 支持 `gx.ai_fun` 语法解析
3. **BlockAction 集成** - 集成到现有的执行框架

### 代码结构
```
galaxy-flow/src/
├── ability/
│   ├── ai_fun.rs          # GxAIFun 实现
│   └── mod.rs            # 能力模块注册
├── parser/
│   ├── inner/
│   │   └── ai_fun.rs     # 解析器实现
│   └── stc_blk.rs        # 块解析器扩展
└── model/
    └── components/
        └── gxl_block.rs   # BlockAction 扩展
```

### 集成点
- 与现有 orion_ai 框架完全兼容
- 复用现有的 AI 客户端和角色系统
- 遵循 Galaxy-flow 的异步执行模型

## 测试覆盖

### 单元测试
- `test_basic_ai_chat` - 测试基础 AI 对话功能
- `test_task_description` - 测试任务描述功能
- `test_empty_task_and_prompt` - 测试默认提示词处理

### 解析器测试
- `ai_fun_role_and_task` - 测试角色和任务参数解析
- `ai_fun_with_prompt` - 测试提示词参数解析
- `ai_fun_minimal` - 测试最小化参数
- `ai_fun_all_params` - 测试所有参数组合

### 集成测试
- 所有示例流程都已通过实际运行验证

## 已知限制

### 第一期限制
1. **工具调用功能** - 尚未实现函数调用机制
2. **任务注册** - 尚未实现任务可重用注册
3. **AI 工作流** - 尚未实现复杂的工作流编排
4. **错误处理** - 基础错误处理，需要增强

### 解码问题
- 某些复杂的 AI 响应可能导致 JSON 解析错误
- 建议使用简洁的提示词避免此问题

## 下一步计划

### 第二期计划
1. 实现工具调用功能
2. 集成 Git 工具函数
3. 增强错误处理机制

### 第三期计划
1. 实现任务注册机制
2. 支持任务重用
3. 增强上下文管理

### 第四期计划
1. 实现 AI 工作流基础
2. 支持简单的工作流编排
3. 任务发现机制

### 第五期计划
1. 完整的智能 Git 工作流
2. 所有功能的整合测试
3. 完善文档和示例

## 故障排除

### 常见错误
1. **API 密钥配置** - 确保已正确配置 AI 服务密钥
2. **网络连接** - 确保能够访问 AI 服务
3. **环境配置** - 确保 gxl 文件中包含正确的环境配置

### 调试技巧
1. 使用 `-d 1` 参数启用调试日志
2. 检查 AI 服务响应内容
3. 验证参数语法是否正确

## 贡献指南

如需贡献代码或报告问题：
1. 遵循现有的代码风格
2. 添加适当的测试用例
3. 更新相关文档

---

**第一期完成时间**: 2025-08-25
**状态**: ✅ 已完成
**测试通过率**: 100%