# Galaxy Flow AI 配置使用指南

> **⚠️ 当前状态：功能已下线，本文档仅作历史留存**
>
> AI 能力目前在 galaxy-flow 中不可用：`orion-ai` 依赖在 `Cargo.toml` 中被注释；`gx.ai_chat` / `gx.ai_fun` 的解析与执行模块未编译（`src/parser/inner/mod.rs`、`src/ability/ai/mod.rs` 中相关 `pub mod` 已注释）；`ai_diagnose` 是 no-op；`gx` 也没有 `ai` 子命令。
>
> 因此本文档中的配置与命令在当前版本**无法生效或执行**。

## 📋 配置文件位置

主配置文件位于：`~/.galaxy/ai.yml`

你可以从示例文件复制配置：
```bash
mkdir -p ~/.galaxy
cp examples/ai_config_example.yml ~/.galaxy/ai.yml
```

## 🔑 环境变量设置

通过安全变量机制 `${SEC_xxx}` 引用 TOKEN（不要把明文密钥写进配置文件）

## ⚙️ 配置原则

示例配置遵循以下设计原则：

### ✅ 推荐做法

```yaml
# 敏感信息使用环境变量
providers:
  openai:
    api_key: "${OPENAI_API_KEY}"

# 非敏感配置使用具体值
providers:
  openai:
    base_url: "https://api.openai.com/v1"
    timeout: 30
    enabled: true
```

### ❌ 不推荐做法

```yaml
# 不要硬编码敏感信息
providers:
  openai:
    api_key: "sk-1234567890abcdef"  # ❌

# 不要全部使用环境变量
providers:
  openai:
    enabled: "${OPENAI_ENABLED}"     # ❌ 直接使用 true 即可
    timeout: "${OPENAI_TIMEOUT}"     # ❌ 直接使用 30 即可
```

## 🔧 变量替换语法

配置文件支持以下环境变量替换语法：

```yaml
# 基本变量替换
api_key: "${OPENAI_API_KEY}"

# 带默认值的替换
timeout: "${OPENAI_TIMEOUT:-30}"

# 必填变量（未设置时会报错）
api_key: "${OPENAI_API_KEY:?}"
```

## 🚀 快速开始

### 1. 设置环境变量
```bash
# 编辑 shell 配置文件
nano ~/.bashrc

# 添加环境变量
export OPENAI_API_KEY="sk-your-real-api-key"
export DEEPSEEK_API_KEY="sk-your-real-deepseek-key"

# 重新加载配置
source ~/.bashrc
```

### 2. 复制配置文件
```bash
cp examples/ai_config_example.yml ~/.galaxy/ai.yml
```

### 3. 验证配置
```bash
# 检查配置文件语法
cat ~/.galaxy/ai.yml

# 验证环境变量
echo $OPENAI_API_KEY
```

### 4. 开始使用

当前版本的 `gx` 没有 `ai` 子命令（见顶部说明），以下历史命令**不可用**，保留仅供参考：

```bash
gflow ai list-models   # 历史命令，当前不可用
gflow ai test          # 历史命令，当前不可用
gflow ai chat          # 历史命令，当前不可用
```

## 🎯 配置优化建议

### 成本优化
```yaml
# 设置 DeepSeek 为默认提供者（成本更低）
routing:
  default_provider: "deepseek"
  cost_optimization: true
```


### 模型别名
```yaml
# 设置简化的模型名称
providers:
  openai:
    model_aliases:
      "gpt4": "gpt-4o"        # 使用 gpt4 时实际调用 gpt-4o
      "latest": "gpt-4o"      # latest 始终指向最新模型
```

## 🐛 常见问题




---

**⚠️ 安全提醒**：不要将包含真实 API Key 的配置文件提交到版本控制系统。确保将 `.galaxy/` 目录添加到 `.gitignore` 文件中。
