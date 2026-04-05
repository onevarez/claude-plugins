# Viewfinder — Claude Code Plugin

Record Playwright browser sessions as cinematic videos with auto-zoom, rounded corners, and polished composition. Works with any Claude Code project that uses Playwright MCP.

## Install

Add the Viewfinder marketplace, then install:

```bash
claude plugins marketplace add onevarez/claude-viewfinder-plugin
claude plugins install viewfinder@viewfinder
```

Restart Claude Code, then run `/vp:setup` to download binaries.

## Usage

```
/vp:setup                    # one-time: downloads binaries, configures Playwright
/vp:record <instructions>    # record a browser session
/vp:stop                     # finalize and produce cinematic video
```

### Examples

```
/vp:record Navigate to stripe.com and explore the pricing page
/vp:record Go to github.com/anthropics/claude-code, click on issues, browse a few
/vp:record Open docs.anthropic.com, search for "tool use", read the first result
```

## What It Does

1. **Records** the Playwright browser session (VP8/WebM via Playwright's `recordVideo`)
2. **Tracks** mouse movements, clicks, and scrolls via an injected cursor tracker
3. **Captures** interaction events from Claude Code hooks
4. **Transcodes** WebM to H.264/MP4
5. **Computes** auto-zoom segments from click clusters
6. **Composes** the final video with rounded corners, dark background, padding, drop shadow, and zoom via [kineto-engine](https://github.com/onevarez/kineto-engine)

## Output

Videos are saved to `~/.viewfinder/sessions/<session-id>/output/cinematic.mp4`.

## Platforms

| Platform | Status |
|---|---|
| macOS arm64 | Supported |
| macOS x64 | Supported |
| Linux x64 | Supported |

## Architecture

Two binaries power the plugin:

- **viewfinder** — session management, hook event capture, manifest/zoom computation, orchestration
- **[kineto](https://github.com/onevarez/kineto-engine)** — cinematic composition (background, corners, shadow, zoom) via statically linked libav*

Both are downloaded automatically by `/vp:setup` from GitHub Releases.

## License

Plugin assets (skills, hooks, scripts): MIT

Composition engine (kineto): [GPL-2.0-or-later](https://github.com/onevarez/kineto-engine/blob/main/LICENSE)
