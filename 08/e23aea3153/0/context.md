# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix Gemini thought_signature round-tripping (#375) and schema validation (#793)

## Context

**#375**: Gemini 3.x models return `thought_signature` on each tool call in their OpenAI-compat
API responses. When Moltis replays tool calls in subsequent turns, this field must be present or
Gemini rejects with HTTP 400. Currently `ToolCall` only carries `id`, `name`, `arguments` -- the
signature is silently dropped at parse time. This affects both direct Gemini API ...

