# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Lazy Tool Registry — Implementation Plan

## Context

Every LLM turn includes full JSON schemas of all registered tools. With a few MCP servers connected this easily burns 15,000+ tokens per turn before the model reads a single user message. PR #330 proposed a solution but was too large (72K additions, mostly unrelated). This plan extracts the core lazy-tool idea and implements it cleanly.

## Design

When `registry_mode = "lazy"` is set in config, the model s...

### Prompt 2

commit and push, create a PR

