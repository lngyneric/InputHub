#!/bin/bash

echo "🚀 发送测试任务..."
curl -s http://127.0.0.1:9527/launch \
  -H 'content-type: application/json' \
  -d '{"raw":"@obsidian 测试","source":"curl","app":"terminal"}'

echo -e "\n\n📋 获取任务列表..."
curl -s http://127.0.0.1:9527/tasks | jq .
