#!/bin/bash
# Download Viewfinder binaries for the current platform.
set -e

VF_DIR="$HOME/.viewfinder"
BIN_DIR="$VF_DIR/bin"
CLI_REPO="onevarez/claude-viewfinder-plugin"
KINETO_REPO="onevarez/kineto-engine"
FFMPEG_BASE_URL="https://github.com/eugeneware/ffmpeg-static/releases/latest/download"

mkdir -p "$BIN_DIR"

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)  PLATFORM="darwin-arm64" ;;
      x86_64) PLATFORM="darwin-x64" ;;
      *)      echo "Error: Unsupported macOS architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64) PLATFORM="linux-x64" ;;
      *)      echo "Error: Unsupported Linux architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*)
    PLATFORM="windows-x64"
    ;;
  *)
    echo "Error: Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

echo "Platform: $PLATFORM"

# ── Download viewfinder-cli ──
echo "Checking latest viewfinder-cli release..."
if command -v gh &>/dev/null; then
  CLI_VERSION=$(gh release list --repo "$CLI_REPO" --limit 5 2>/dev/null | grep "^v" | head -1 | awk '{print $3}')
else
  CLI_VERSION=$(curl -sL "https://api.github.com/repos/$CLI_REPO/releases/latest" | python3 -c "import sys,json; print(json.load(sys.stdin).get('tag_name',''))" 2>/dev/null)
fi

if [ -n "$CLI_VERSION" ]; then
  echo "viewfinder-cli: $CLI_VERSION"
  ARCHIVE="viewfinder-${PLATFORM}.tar.gz"
  TMPDIR=$(mktemp -d)

  if command -v gh &>/dev/null; then
    gh release download "$CLI_VERSION" --repo "$CLI_REPO" --pattern "$ARCHIVE" --dir "$TMPDIR" 2>/dev/null
  else
    curl -sL "https://github.com/$CLI_REPO/releases/download/$CLI_VERSION/$ARCHIVE" -o "$TMPDIR/$ARCHIVE"
  fi

  if [ -s "$TMPDIR/$ARCHIVE" ]; then
    tar -xzf "$TMPDIR/$ARCHIVE" -C "$BIN_DIR"
    chmod +x "$BIN_DIR/viewfinder" 2>/dev/null || true
  else
    echo "Warning: viewfinder-cli download failed" >&2
  fi
  rm -rf "$TMPDIR"
else
  echo "Warning: No viewfinder-cli releases found" >&2
fi

# ── Download kineto (compositor) ──
echo "Checking latest kineto release..."
if command -v gh &>/dev/null; then
  KINETO_VERSION=$(gh release list --repo "$KINETO_REPO" --limit 5 2>/dev/null | grep "^v" | head -1 | awk '{print $3}')
else
  KINETO_VERSION=$(curl -sL "https://api.github.com/repos/$KINETO_REPO/releases/latest" | python3 -c "import sys,json; print(json.load(sys.stdin).get('tag_name',''))" 2>/dev/null)
fi

if [ -n "$KINETO_VERSION" ]; then
  echo "kineto: $KINETO_VERSION"
  ARCHIVE="kineto-${PLATFORM}.tar.gz"
  TMPDIR=$(mktemp -d)

  if command -v gh &>/dev/null; then
    gh release download "$KINETO_VERSION" --repo "$KINETO_REPO" --pattern "$ARCHIVE" --dir "$TMPDIR" 2>/dev/null
  else
    curl -sL "https://github.com/$KINETO_REPO/releases/download/$KINETO_VERSION/$ARCHIVE" -o "$TMPDIR/$ARCHIVE"
  fi

  if [ -s "$TMPDIR/$ARCHIVE" ]; then
    tar -xzf "$TMPDIR/$ARCHIVE" -C "$BIN_DIR"
    chmod +x "$BIN_DIR/kineto" 2>/dev/null || true
  else
    echo "Warning: kineto download failed" >&2
  fi
  rm -rf "$TMPDIR"
else
  echo "Warning: No kineto releases found" >&2
fi

# ── Download ffmpeg/ffprobe (for WebM → MP4 transcode) ──
if [ ! -f "$BIN_DIR/ffmpeg" ]; then
  echo "Downloading ffmpeg..."
  case "$PLATFORM" in
    darwin-arm64)  FFMPEG_TARGET="darwin-arm64" ;;
    darwin-x64)    FFMPEG_TARGET="darwin-x64" ;;
    linux-x64)     FFMPEG_TARGET="linux-x64" ;;
    windows-x64)   FFMPEG_TARGET="win32-x64" ;;
  esac
  curl -sL "$FFMPEG_BASE_URL/ffmpeg-$FFMPEG_TARGET" -o "$BIN_DIR/ffmpeg"
  chmod +x "$BIN_DIR/ffmpeg"

  curl -sL "$FFMPEG_BASE_URL/ffprobe-$FFMPEG_TARGET" -o "$BIN_DIR/ffprobe"
  chmod +x "$BIN_DIR/ffprobe"
fi

# ── Write version file ──
cat > "$VF_DIR/version.json" << EOF
{
  "cli_version": "${CLI_VERSION:-unknown}",
  "kineto_version": "${KINETO_VERSION:-unknown}",
  "platform": "$PLATFORM",
  "installed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "bin_dir": "$BIN_DIR"
}
EOF

# ── Generate Playwright config ──
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$SCRIPT_DIR/../config"
TRACKER_PATH="$(cd "$CONFIG_DIR" && pwd)/tracker.js"

cat > "$CONFIG_DIR/playwright-config.json" << PEOF
{
  "browser": {
    "isolated": true,
    "contextOptions": {
      "viewport": { "width": 1920, "height": 1080 },
      "deviceScaleFactor": 1,
      "recordVideo": {
        "dir": "./playwright-videos/",
        "size": { "width": 1920, "height": 1080 }
      }
    },
    "initScript": ["$TRACKER_PATH"]
  },
  "capabilities": ["core", "vision", "devtools"],
  "outputDir": "./playwright-output",
  "outputMode": "file",
  "consoleLevel": "info"
}
PEOF

# ── Add to PATH ──
for profile in "$HOME/.zprofile" "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile"; do
  if [ -f "$profile" ] || [ "$profile" = "$HOME/.zprofile" ]; then
    if ! grep -q '.viewfinder/bin' "$profile" 2>/dev/null; then
      echo 'export PATH="$HOME/.viewfinder/bin:$PATH"' >> "$profile"
    fi
  fi
done

echo ""
echo "Installed for $PLATFORM"
echo "  viewfinder:         $BIN_DIR/viewfinder"
echo "  kineto:             $BIN_DIR/kineto"
echo "  ffmpeg:             $BIN_DIR/ffmpeg"
echo "  playwright-config:  $CONFIG_DIR/playwright-config.json"
echo "  tracker.js:         $TRACKER_PATH"
