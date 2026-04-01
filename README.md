# Agent Hub Starter（Phase 1→3 可落地骨架）

这份 starter code 按你整理的三阶段路线做了“能跑起来”的最小闭环骨架：

- **Phase 1**：Rime 发射（HTTP/Unix Socket 二选一）→ Hub 接收 → SQLite 落库 → 日志可见
- **Phase 2**：内置第一个本地 Agent：`@obsidian`（ripgrep 搜索 vault）
- **Phase 3**：LLM 路由层骨架：`router` + `doubao`（以及 `claude` 占位），支持“简单/复杂”分流与（预留）流式输出接口

> 说明：这里提供的是**可直接编译运行的 Rust Hub**（先不强依赖 Tauri UI）。你可以把 `src-tauri` 下的代码直接搬进后续 `tauri-app` 模板里（在 `main.rs` 里 `tauri::Builder` 启动时 spawn 这些服务即可）。

---

## 目录

```
agent-hub-starter/
  rime-plugin/
    lua/agent_launcher.lua
    install.sh
  src-tauri/
    Cargo.toml
    src/
      main.rs
      config.rs
      types.rs
      db.rs
      ipc/
        mod.rs
        http.rs
        unix_socket.rs
      agents/
        mod.rs
        obsidian.rs
        deerflow.rs
      llm/
        mod.rs
        router.rs
        doubao.rs
        claude.rs
```

---

## Phase 1：先跑起来（Hub）

### 1) 编译运行

```bash
cd agent-hub-starter/src-tauri
cargo run
```

默认会：
- 启动 HTTP：`127.0.0.1:9527`
  - `POST /launch`：接收任务
  - `GET  /tasks`：查看任务列表
- 启动 Unix Socket：`/tmp/agent_hub.sock`（macOS/Linux）
- SQLite：默认 `~/.agent-hub/agent_hub.sqlite`（可用环境变量覆盖，见下）

### 2) 用 curl 发射一条任务（验证闭环）

```bash
curl -s http://127.0.0.1:9527/launch \
  -H 'content-type: application/json' \
  -d '{"raw":"@test hello","source":"curl","app":"terminal"}'
```

预期：终端日志出现 `received`，并且 `GET /tasks` 能看到记录：

```bash
curl -s http://127.0.0.1:9527/tasks | jq .
```

---

## Phase 1：Rime 发射器（最小可用）

`rime-plugin/lua/agent_launcher.lua` 给了一个**可复制进 Rime** 的 Lua processor：
- 检测 `@agent ...` 输入
- 在你按下 `Space/Return` 时**发射任务**（默认走 HTTP `127.0.0.1:9527/launch`，用 `curl`）
- 清空 context（不把 `@xxx` 写进当前应用）

> “✓ 已发射”提示：Rime 本身缺少统一 toast API，不同发行版行为差异较大。starter 里默认用 **macOS `osascript` 通知（可关）**；你也可以替换成自己的 UI 提示方式。

---

## 配置（环境变量）

Hub 运行时可用环境变量覆盖默认值：

- `AGENT_HUB_DB_PATH`：SQLite 路径（默认 `~/.agent-hub/agent_hub.sqlite`）
- `AGENT_HUB_HTTP_ADDR`：HTTP 监听地址（默认 `127.0.0.1:9527`）
- `AGENT_HUB_SOCKET_PATH`：Unix Socket 路径（默认 `/tmp/agent_hub.sock`）
- `OBSIDIAN_VAULT_PATH`：Obsidian vault 目录（Phase 2 必填）
- `DOUBAO_API_KEY` / `DOUBAO_BASE_URL` / `DOUBAO_MODEL`：豆包路由层（Phase 3）
- `CLAUDE_API_KEY`：Claude（占位）

---

## 下一步怎么接 Tauri UI？

建议做法：
1. 用官方模板创建 tauri 项目（React/TS）
2. 把本目录 `src-tauri/src/*` 的模块合并进去
3. 在 tauri `main.rs` 启动时 `tokio::spawn`：HTTP / Unix Socket 服务
4. 前端用 `tauri::invoke` 拉取 `/tasks` 或直接把任务事件通过 WebSocket（下一步再加）推到 UI

