# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix: OpenClaw import overwrites config template comments (GH-458)

## Context

When a user does a fresh install and imports from OpenClaw, the commented config examples in `moltis.toml` are lost. This happens because:

1. Server starts → `discover_and_load()` → no config file → `write_default_config()` writes the **full template** with all commented examples (~700 lines of documented options)
2. User goes through onboarding → clicks "Import from OpenClaw"
3. `...

### Prompt 2

commit, push, create a PR

