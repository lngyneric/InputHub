-- Agent Hub Rime Launcher (starter)
--
-- 功能：
-- - 拦截以 @ 开头的指令（例如：@test hello、@obsidian 供应链培训方案）
-- - 在你按下回车时，把整段 raw 发射到本地 Hub（默认 HTTP 127.0.0.1:9527/launch）
-- - 清空输入，不把 @xxx 写进当前应用
--
-- 依赖：
-- - macOS/Linux：需要系统有 curl（默认有）。Windows 发行版可改成 powershell Invoke-WebRequest。
--
-- 使用：
-- 1) 把本文件放到 Rime 用户目录：lua/agent_launcher.lua
-- 2) 在你的 schema 里加入：
--    engine:
--      processors:
--        - lua_processor@agent_launcher

local kRejected = 0
local kAccepted = 1
local kNoop = 2

local function json_escape(s)
  s = s:gsub("\\", "\\\\")
  s = s:gsub("\"", "\\\"")
  s = s:gsub("\n", "\\n")
  s = s:gsub("\r", "\\r")
  s = s:gsub("\t", "\\t")
  return s
end

local function sh_single_quote(s)
  -- wrap into single quotes, escape internal single quote safely: ' -> '"'"'
  return "'" .. s:gsub("'", "'\"'\"'") .. "'"
end

local function emit_via_http(raw, hub_url)
  local body = string.format('{"raw":"%s","source":"rime","app":""}', json_escape(raw))
  local cmd = string.format(
    "curl -sS -m 0.25 -X POST %s -H 'content-type: application/json' -d %s >/dev/null 2>&1",
    sh_single_quote(hub_url),
    sh_single_quote(body)
  )
  os.execute(cmd)
end

local function mac_notify(msg)
  -- 可选：不侵入输入的提示（可删除）
  local cmd = string.format(
    "osascript -e %s >/dev/null 2>&1",
    sh_single_quote(string.format('display notification "%s" with title "Agent Hub"', msg))
  )
  os.execute(cmd)
end

local function should_fire_at(raw)
  if not raw then return false end
  raw = raw:gsub("^%s+", "")
  return raw:match("^@%S+") ~= nil
end

local function is_enter(key)
  local r = key:repr()
  return r == "Return" or r == "KP_Enter"
end

local function is_space(key)
  local r = key:repr()
  return r == "space" or r == "Space"
end

local function selected_candidate_text(ctx)
  local ok, cand = pcall(function()
    return ctx:get_selected_candidate()
  end)
  if not ok or not cand then
    return nil
  end
  return cand.text
end

local function agent_launcher(key, env)
  local ctx = env.engine.context
  local input = ctx.input or ""
  local hub_url = env.hub_url or "http://127.0.0.1:9527/launch"

  -- 1) aa -> 进入 AI 模式（选择置顶 ai:）
  local r = key:repr()
  local choose_first = is_space(key) or r == "1"
  if choose_first and input == "aa" and not ctx:get_option("ai_mode") then
    local sel = selected_candidate_text(ctx)
    if sel ~= "ai:" then
      return kNoop
    end
    ctx:set_option("ai_mode", true)
    ctx:clear()
    mac_notify("AI 模式已启用")
    return kAccepted
  end

  -- 2) AI 模式：Enter 截断并发送；应用里落字关键词（不含 ai:）
  if ctx:get_option("ai_mode") then
    if is_enter(key) then
      local keyword = input
      local raw = "@obsidian " .. keyword
      emit_via_http(raw, hub_url)
      env.engine:commit_text(keyword)
      ctx:clear()
      ctx:set_option("ai_mode", false)
      mac_notify("✓ 已发射")
      return kAccepted
    end

    local r = key:repr()
    if r == "Escape" then
      ctx:set_option("ai_mode", false)
      mac_notify("已退出 AI 模式")
      return kAccepted
    end

    return kNoop
  end

  -- 3) 兼容原有 @ 开头触发：Enter 发射后清空，不落字
  if should_fire_at(input) then
    if is_enter(key) or key:repr() == "Shift+A" then
      emit_via_http(input, hub_url)
      ctx:clear()
      mac_notify("✓ 已发射")
      return kAccepted
    else
      return kNoop
    end
  end

  return kNoop
end

return agent_launcher
