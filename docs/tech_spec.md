# Agent Hub 项目开发规范与技术文档

## 1. 项目概述

Agent Hub 是一个基于本地常驻服务（Hub）和输入法拦截（Rime 插件）的“零摩擦”个人 AI 工作流中枢系统。
项目的核心目标是通过全局输入法拦截，随时随地将用户指令路由到本地知识库检索（如 Obsidian）或云端大模型分析（如豆包、Claude），并最终通过 Tauri UI 呈现。

### 1.1 核心架构
系统分为三个主要层次：
1. **输入层（Input Layer）**：Rime Lua 插件（`agent_launcher.lua`），负责在用户输入以 `@` 开头的指令时拦截、清空并发送请求。
2. **中枢层（Hub Layer）**：Rust 编写的本地常驻服务（基于 `tokio` + `axum`），负责接收任务、意图识别、路由分发与状态管理（SQLite）。
3. **展现层（UI Layer）**：未来通过 Tauri 构建的前端应用，负责监听任务状态并弹出卡片展示结果。

---

## 2. 目录结构说明

```text
agent-hub/
├── README.md                 # 项目快速启动说明
├── test_hub.sh               # 快速测试脚本
├── docs/                     # 项目文档目录
│   └── tech_spec.md          # 本文档：开发规范与技术设计
├── rime-plugin/              # 鼠须管/Rime 输入法插件目录
│   └── lua/
│       └── agent_launcher.lua # 输入拦截脚本
└── src-tauri/                # 后端中枢（Rust + Tauri）
    ├── Cargo.toml            # Rust 依赖配置
    └── src/
        ├── main.rs           # 服务入口与初始化
        ├── config.rs         # 环境变量与配置解析
        ├── types.rs          # 核心数据结构（任务、状态）
        ├── db.rs             # SQLite 数据库操作
        ├── ipc/              # 进程间通信模块（HTTP/Unix Socket）
        │   ├── mod.rs
        │   ├── http.rs       # Axum HTTP 服务
        │   └── unix_socket.rs# Unix Socket 服务（macOS/Linux）
        ├── llm/              # 大模型与路由层
        │   ├── mod.rs
        │   ├── router.rs     # 意图识别分发（简单/复杂）
        │   ├── doubao.rs     # 豆包 API 对接
        │   └── claude.rs     # Claude API 对接（占位）
        └── agents/           # 具体任务执行代理
            ├── mod.rs        # 代理调度入口
            ├── obsidian.rs   # 本地 Obsidian 检索代理
            └── deerflow.rs   # 复杂分析代理（占位）
```

---

## 3. 开发规范

### 3.1 编码规范 (Rust)
- **代码风格**：统一使用 `cargo fmt` 进行格式化。
- **错误处理**：
  - 核心业务逻辑必须返回 `anyhow::Result<T>`，禁止直接使用 `unwrap()` 或 `expect()` 导致进程 Panic。
  - 使用 `anyhow::Context` 为错误添加上下文，例如 `.context("failed to parse config")?`。
- **异步编程**：
  - 统一使用 `tokio` 运行时。
  - Agent 的执行必须放入 `tokio::spawn` 独立运行，不可阻塞主 HTTP/Socket 线程。
- **日志规范**：
  - 统一使用 `tracing` 宏（`info!`, `warn!`, `error!`, `debug!`）。
  - 日志中必须附带关键变量，例如：`info!(task_id = %id, "task started")`。

### 3.2 状态机管理
任务 (`TaskRecord`) 必须严格遵循以下状态流转 (`TaskStatus`)：
1. `Received`：HTTP 接口接收到任务，成功写入 SQLite。
2. `Running`：进入 `agents::execute_task`，开始处理。
3. `Succeeded` / `Failed`：任务完成或报错，更新 SQLite 并存储 Markdown 结果与元数据。

---

## 4. 扩展指南：如何新增一个 Agent

当需要接入新的能力（如：`@github` 查询 Issues）时，按以下步骤操作：

1. **创建 Agent 文件**：在 `src-tauri/src/agents/` 下新建 `github.rs`。
2. **实现 `run` 函数**：
   ```rust
   pub async fn run(task_id: Uuid, query: &str, cfg: &Config) -> anyhow::Result<(String, serde_json::Value)> {
       // 1. 调用 API
       // 2. 组装 Markdown 结果
       // 3. 返回 (Markdown字符串, 结构化元数据)
   }
   ```
3. **注册 Agent**：在 `src-tauri/src/agents/mod.rs` 中：
   - 添加 `pub mod github;`
   - 在 `execute_task` 的路由逻辑中补充 `raw.starts_with("@github")` 分支。
4. **补充环境变量**（如需）：在 `config.rs` 中添加对应 Key，例如 `GITHUB_TOKEN`。

---

## 5. 核心环境变量清单

项目强依赖环境变量进行配置注入，推荐使用 `.env` 文件或在启动前 `export`：

| 变量名 | 默认值 | 描述 |
|---|---|---|
| `AGENT_HUB_DB_PATH` | `~/.agent-hub/agent_hub.sqlite` | SQLite 数据库存储路径 |
| `AGENT_HUB_HTTP_ADDR` | `127.0.0.1:9527` | HTTP 监听地址 |
| `OBSIDIAN_VAULT_PATH` | *(空)* | (必需) Obsidian 仓库绝对路径 |
| `DOUBAO_API_KEY` | *(空)* | 豆包大模型 API Key（用于路由分发） |
| `DOUBAO_MODEL` | `ep-20241225134106-kpsps` | 豆包大模型接入点 ID |

---

## 6. 环境依赖与启动

项目依赖基础系统工具与 Rust 构建工具链。详细的安装与环境补充见辅助安装脚本。
主要外部依赖：
- **Rust 编译链**：`cargo`，`rustc` (>= 1.70)
- **系统命令**：`rg` (ripgrep) 用于本地极速文件搜索，`curl` 用于测试与发送请求。
