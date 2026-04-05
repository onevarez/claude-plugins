# Viewfinder Stop — Finalize recording and produce cinematic video

## Usage

```
/viewfinder:stop
```

## Allowed tools

- `mcp__playwright__*`
- `Bash(viewfinder session *)`

## Instructions

1. Call `mcp__playwright__browser_close`
2. Run:

```bash
viewfinder session finalize ACTIVE_SESSION_ID
```

Report the output path.

## CRITICAL RULES

- **NEVER build compound Bash commands.** Only call `viewfinder` with subcommands.
