# Session Context

## User Prompts

### Prompt 1

See if you can figure https://github.com/moltis-org/moltis/issues/766 out and plan for a fix

### Prompt 2

proceed

### Prompt 3

commit push create a PR

### Prompt 4

I feel like this should be behind the slack feature flag:

// Slack slash command webhook -- receives /command payloads.
        let slack_cmd_plugin = Arc::clone(&slack_webhook_plugin);
        let state_for_slack_cmd = Arc::clone(&state);
        app = app.route(

### Prompt 5

Base directory for this skill: /Users/penso/.claude/skills/greploop

# Greploop

Iteratively fix a PR/MR/CL until Greptile gives a perfect review: 5/5 confidence, zero unresolved comments.

## Inputs

- **PR/MR/CL number** (optional): If not provided, detect the PR/MR for the current branch, or the default pending changelist for p4.

## Instructions

### 0. Detect platform

First check for Perforce, then fall back to git remote detection:

```bash
# Check for Perforce environment
if p4 info >...

