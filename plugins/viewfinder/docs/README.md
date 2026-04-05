# Viewfinder

Cinematic browser session recording for Claude Code. Captures Playwright sessions as polished videos with auto-zoom, rounded corners, background, and drop shadow.

## Install

```bash
claude plugins marketplace add onevarez/claude-plugins
claude plugins install viewfinder@onevarez
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

## How It Works

Records the Playwright browser session as video (VP8/WebM), tracks mouse movements and clicks via an injected cursor tracker, captures interaction events through Claude Code hooks, computes auto-zoom segments from click clusters, and composes the final video with cinematic treatment via [kineto-engine](https://github.com/onevarez/kineto-engine).

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

Composition engine ([kineto](https://github.com/onevarez/kineto-engine)): GPL-2.0-or-later
