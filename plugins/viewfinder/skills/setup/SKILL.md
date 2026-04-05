# Viewfinder Setup — Install Viewfinder

Download binaries, configure Playwright MCP, and set up video recording. Run once per project.

## Usage

```
/vp:setup
```

## Allowed tools

- `mcp__playwright__*`
- `Bash(${CLAUDE_PLUGIN_ROOT}/scripts/download-binaries.sh)`
- `Write`
- `Read`

## Instructions

### Step 1: Download binaries

```bash
${CLAUDE_PLUGIN_ROOT}/scripts/download-binaries.sh
```

This downloads `viewfinder` and `kineto` to `~/.viewfinder/bin/`. It also adds `~/.viewfinder/bin` to the user's shell profiles automatically.

### Step 2: Add Playwright MCP to this project

Check if `.mcp.json` exists in the current directory. Read it if so. If it already has a `playwright` entry, skip.

Otherwise, use the Write tool to create `.mcp.json`. Use the `playwright-config:` path from the download script output.

Example `.mcp.json`:
```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["@playwright/mcp@latest", "--config", "PLAYWRIGHT_CONFIG_PATH"]
    }
  }
}
```

Replace `PLAYWRIGHT_CONFIG_PATH` with the actual path from the download output.

### Step 3: Report

Tell the user exactly this:

> **Setup complete.** Close your terminal, reopen it, then start Claude Code again. This is needed once so the `viewfinder` command and Playwright MCP are available.
>
> After restarting, use `/viewfinder:record <instructions>` to record browser sessions.
