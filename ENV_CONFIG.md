# AI Chatbot 环境变量配置说明

本文档详细说明了项目中所有环境变量的用途和配置方法。

## 快速开始

1. 复制示例配置文件：

```bash
cp .env.example .env
```

2. 根据你的需求编辑 `.env` 文件

## 配置项详解

### LLM Provider（大语言模型提供商）

#### `LLM_PROVIDER`

- **说明**：选择使用的 LLM 提供商
- **可选值**：`mock`, `deepseek`, `doubao`, `grok`
- **默认值**：`mock`
- **示例**：`LLM_PROVIDER=doubao`

---

### DeepSeek 配置

#### `DEEPSEEK_API_KEY`

- **说明**：DeepSeek API 密钥
- **必需**：当 `LLM_PROVIDER=deepseek` 时
- **获取方式**：访问 [DeepSeek 官网](https://platform.deepseek.com/)

#### `DEEPSEEK_MODEL`

- **说明**：使用的 DeepSeek 模型名称
- **默认值**：`deepseek-chat`
- **可选值**：`deepseek-chat`, `deepseek-coder` 等

---

### Doubao/Ark 配置（字节跳动火山引擎）

#### `DOUBAO_API_KEY`

- **说明**：Doubao/Ark API 密钥
- **必需**：当 `LLM_PROVIDER=doubao` 时
- **获取方式**：访问[火山引擎控制台](https://console.volcengine.com/)

#### `DOUBAO_MODEL`

- **说明**：聊天模型的 Endpoint ID
- **必需**：当 `LLM_PROVIDER=doubao` 时
- **格式**：通常是一串字母数字组合
- **示例**：`ep-20240101-xxxxx`

#### `DOUBAO_VISION_MODEL`（多模态支持）

- **说明**：视觉模型的 Endpoint ID，用于图片/视频分析
- **可选**：启用视觉功能时需要
- **功能**：
  - 图片内容理解
  - 视频内容分析（待完善）

#### `DOUBAO_ASR_MODEL`（语音识别）

- **说明**：语音识别模型的 Endpoint ID
- **可选**：启用语音转文字功能时需要
- **状态**：架构已就绪，功能待实现

#### `DOUBAO_TTS_MODEL`（语音合成）

- **说明**：语音合成模型的 Endpoint ID
- **可选**：启用文字转语音功能时需要
- **状态**：架构已就绪，功能待实现

---

### Grok/xAI 配置

#### `GROK_API_KEY`

- **说明**：Grok API 密钥
- **必需**：当 `LLM_PROVIDER=grok` 时
- **获取方式**：访问 [xAI 官网](https://x.ai/)

#### `GROK_MODEL`

- **说明**：使用的 Grok 模型名称
- **默认值**：`grok-beta`

---

### Memory（记忆/上下文管理）

#### `MEMORY_TYPE`

- **说明**：选择记忆存储方式
- **可选值**：
  - `none`：无持久化存储（仅当前会话）
  - `postgres`：PostgreSQL 数据库存储
  - `redis`：Redis 缓存存储
- **默认值**：`none`

---

### PostgreSQL 配置

#### `DATABASE_URL`

- **说明**：PostgreSQL 数据库连接字符串
- **必需**：当 `MEMORY_TYPE=postgres` 时
- **格式**：`postgres://username:password@host:port/database`
- **示例**：`postgres://chatbot:secret@localhost:5432/chatbot_db`

**数据库初始化**：

```sql
CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    user_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_session_id ON messages(session_id);
CREATE INDEX idx_created_at ON messages(created_at);
```

---

### Redis 配置

#### `REDIS_URL`

- **说明**：Redis 连接 URL
- **必需**：当 `MEMORY_TYPE=redis` 时
- **格式**：`redis://[username:password@]host:port[/database]`
- **示例**：
  - `redis://127.0.0.1/`（本地无密码）
  - `redis://:password@127.0.0.1:6379/0`（带密码）

---

### Platform（平台适配器）

#### `PLATFORM`

- **说明**：选择运行平台
- **可选值**：
  - `terminal`：终端交互模式
  - `onebot`：OneBot 协议（QQ/微信等）
- **默认值**：`terminal`

---

### OneBot 配置

#### `ONEBOT_WS_URL`

- **说明**：OneBot WebSocket 服务器地址
- **必需**：当 `PLATFORM=onebot` 时
- **格式**：`ws://host:port`
- **示例**：`ws://127.0.0.1:6700`
- **兼容**：
  - go-cqhttp
  - Lagrange
  - 其他 OneBot v11 实现

---

### 日志配置

#### `RUST_LOG`

- **说明**：控制日志输出级别
- **可选**：不设置则使用默认级别
- **格式**：`[module=]level[,...]`
- **级别**：`trace`, `debug`, `info`, `warn`, `error`
- **示例**：
  - `RUST_LOG=info`：全局 info 级别
  - `RUST_LOG=chatbot=debug,info`：chatbot 模块 debug，其他 info
  - `RUST_LOG=debug`：全局 debug 级别

---

## 配置示例

### 示例 1：本地开发（Mock LLM + 终端）

```bash
LLM_PROVIDER=mock
MEMORY_TYPE=none
PLATFORM=terminal
RUST_LOG=debug
```

### 示例 2：DeepSeek + PostgreSQL + 终端

```bash
LLM_PROVIDER=deepseek
DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxx
DEEPSEEK_MODEL=deepseek-chat

MEMORY_TYPE=postgres
DATABASE_URL=postgres://chatbot:password@localhost/chatbot

PLATFORM=terminal
RUST_LOG=info
```

### 示例 3：Doubao 多模态 + Redis + OneBot

```bash
LLM_PROVIDER=doubao
DOUBAO_API_KEY=your_api_key
DOUBAO_MODEL=ep-20240101-chat
DOUBAO_VISION_MODEL=ep-20240101-vision

MEMORY_TYPE=redis
REDIS_URL=redis://127.0.0.1/

PLATFORM=onebot
ONEBOT_WS_URL=ws://127.0.0.1:6700

RUST_LOG=chatbot=debug,info
```

---

## 注意事项

1. **API 密钥安全**：

   - 不要将 `.env` 文件提交到版本控制系统
   - 已在 `.gitignore` 中排除 `.env` 文件

2. **Endpoint ID 获取**（Doubao）：

   - 登录火山引擎控制台
   - 进入"大模型服务" → "火山方舟"
   - 在"API 接入"中创建或查看 Endpoint ID

3. **数据库准备**：

   - 使用 PostgreSQL 前需先创建数据库和表
   - 使用 Redis 前需确保 Redis 服务已启动

4. **OneBot 配置**：
   - 需要先配置好 OneBot 实现（如 go-cqhttp）
   - WebSocket 地址需与 OneBot 配置一致

---

## 故障排查

### 问题：API 调用失败

- 检查 API 密钥是否正确
- 检查网络连接
- 查看 `RUST_LOG=debug` 输出的详细日志

### 问题：数据库连接失败

- 确认数据库服务已启动
- 检查 `DATABASE_URL` 格式是否正确
- 确认数据库用户有足够权限

### 问题：OneBot 连接失败

- 确认 OneBot 服务已启动
- 检查 WebSocket URL 是否正确
- 查看 OneBot 日志输出
