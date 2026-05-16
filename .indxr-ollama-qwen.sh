#!/bin/bash
# Wrapper for indxr wiki → Ollama OpenAI-compatible API (qwen3.6:27b)
input=$(cat)

# Handle both {"prompt": "..."} and {"messages": [...]} formats
messages=$(echo "$input" | python3 -c "
import sys, json
data = json.load(sys.stdin)
if 'messages' in data:
    print(json.dumps(data['messages']))
elif 'prompt' in data:
    print(json.dumps([{'role': 'user', 'content': data['prompt']}])
)
else:
    print(json.dumps([{'role': 'user', 'content': str(data)}])
)")

response=$(curl -s http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d "{\"model\": \"qwen3.6:27b\", \"messages\": $messages, \"stream\": false}")

echo "$response" | python3 -c "
import sys, json
data = json.load(sys.stdin)
if 'choices' in data:
    content = data['choices'][0]['message'].get('content', '')
    if not content:
        content = data['choices'][0]['message'].get('reasoning', '')
    print(content)
elif 'error' in data:
    print(f'Error: {data[\"error\"]}', file=sys.stderr)
    sys.exit(1)
else:
    print(json.dumps(data))
"
