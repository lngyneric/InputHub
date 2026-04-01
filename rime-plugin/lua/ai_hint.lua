-- AI Hint Filter: 在输入为 "aa" 时，将 "ai:" 插入为首位候选以进入 AI 模式

local function ai_hint(input, env)
  local ctx = env.engine.context
  local in_ai = ctx:get_option("ai_mode")
  local preedit = ctx.input or ""

  if not in_ai and preedit == "aa" then
    local cand = Candidate("ai_hint", 0, 0, "ai:", "进入 AI 搜索模式")
    cand.quality = 100.0
    yield(cand)
  end

  for cand in input:iter() do
    yield(cand)
  end
end

return ai_hint
