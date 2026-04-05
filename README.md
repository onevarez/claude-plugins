# onevarez — Claude Code Plugins

A marketplace of Claude Code plugins. Add it once, install any plugin by name.

```bash
claude plugins marketplace add onevarez/claude-plugins
```

## Plugins

### Viewfinder

Cinematic browser session recording. Captures Playwright sessions as polished videos with auto-zoom, rounded corners, background, and drop shadow.

```bash
claude plugins install viewfinder@onevarez
```

After restarting Claude Code, run `/vp:setup` to download binaries.

```
/vp:setup                    # one-time: downloads binaries, configures Playwright
/vp:record <instructions>    # record a browser session
/vp:stop                     # finalize and produce cinematic video
```

**Examples:**

```
/vp:record Navigate to stripe.com and explore the pricing page
/vp:record Go to github.com/anthropics/claude-code, click on issues, browse a few
```

**How it works:** Records the Playwright browser session as video, tracks mouse movements and clicks, captures interaction events via hooks, computes auto-zoom segments from click clusters, and composes the final video with cinematic treatment via [kineto-engine](https://github.com/onevarez/kineto-engine).

**Output:** `~/.viewfinder/sessions/<session-id>/output/cinematic.mp4`

**Platforms:** macOS arm64, macOS x64, Linux x64

**Binaries:** `viewfinder` (orchestration) + [`kineto`](https://github.com/onevarez/kineto-engine) (composition via static libav*). Both downloaded by `/vp:setup`.

---

## License

Plugin assets (skills, hooks, scripts): MIT

Composition engine (kineto): [GPL-2.0-or-later](https://github.com/onevarez/kineto-engine/blob/main/LICENSE)
