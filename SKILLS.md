# Agent skills

This repository ships the **build-metassr-app** skill: a harness-agnostic
workflow for building a MetaSSR app from a user's idea — intake questions,
scaffold, build, run, iterate.

## In this repo

The skill lives in `.opencode/skills/build-metassr-app/`. It is symlinked into
`.claude/skills/build-metassr-app` and `.codex/skills/build-metassr-app`, so
opencode, Claude Code and Codex all pick it up automatically when working in a
checkout of this repo. The symlinks are committed; do not replace them with
copies.

| Harness | Repo location (symlink) |
| --- | --- |
| opencode | `.opencode/skills/build-metassr-app/` |
| Claude Code | `.claude/skills/build-metassr-app` |
| Codex | `.codex/skills/build-metassr-app` |

## Personal install (any project, any machine)

Copy the skill folder into your harness's global skill directory:

```sh
# opencode
cp -r .opencode/skills/build-metassr-app ~/.config/opencode/skills/

# Claude Code
cp -r .opencode/skills/build-metassr-app ~/.claude/skills/

# Codex
cp -r .opencode/skills/build-metassr-app ~/.codex/skills/

# open standard (~/.agents/skills is picked up by opencode, and by other
# harnesses that follow the agents.md convention)
cp -r .opencode/skills/build-metassr-app ~/.agents/skills/
```

Restart the harness after installing. Then prompt, e.g.:

> build me a dashboard app with cards and a settings page

## Note for contributors

Keep the skill self-contained and harness-agnostic (plain markdown, no
harness-specific directives). The repo's example-gallery and container flows
belong in `AGENTS.md`, not in the skill.