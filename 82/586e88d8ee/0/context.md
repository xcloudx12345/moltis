# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: Auto-Continue on Model Stop + Max Iterations UX

## Context

A user reported that Moltis "just stops" mid-task with no explanation. Their log showed:
```
agent run complete iterations=12 tool_calls=11
```

**Diagnosis:** The log says `"agent run complete"` (success path), not `"agent loop exceeded max iterations"` (error path). The model chose to stop at iteration 12 — it returned text without tool calls. The default max is 25, so there was budget remain...

### Prompt 2

maybe `MAX_AUTO_CONTINUES = 2` should also be a config flag for user to change that

### Prompt 3

commit push and create a PR

### Prompt 4

## Context

- Current git status: On branch bubble-carp
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   crates/agents/src/runner.rs
	modified:   crates/chat/src/chat_error.rs
	modified:   crates/chat/src/lib.rs
	modified:   crates/config/src/schema.rs
	modified:   crates/config/src/template.rs
	modified:   crates/config/src/validate.rs
	modified:   crates/web/src/as...

### Prompt 5

Fix PR comments

