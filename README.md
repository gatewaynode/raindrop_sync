# raindrop_sync

Syncs all your [Raindrop.io](https://raindrop.io) bookmarks to a local `bookmarks.json` file.

## Setup

### 1. Get a Raindrop API token

1. Go to [Raindrop Settings → Integrations](https://app.raindrop.io/settings/integrations)
2. Create a new app (or use an existing one)
3. Copy the **Test token** — no OAuth flow required

### 2. Build the binary

```bash
git clone <repo>
cd raindrop_sync
cargo build --release
```

The binary will be at `target/release/raindrop_sync`.

### 3. Configure

On first run, `raindrop_sync` automatically creates a config file at:

```
~/.config/raindrop_sync/config.toml
```

(or `$XDG_CONFIG_HOME/raindrop_sync/config.toml` if `XDG_CONFIG_HOME` is set)

Open the file and add your API key — the app will not run until it is set:

```toml
# Raindrop.io API key.
# Get your test token from: https://app.raindrop.io/settings/integrations
# The RAINDROP_TOKEN environment variable takes precedence if set.
api_key = "paste_your_token_here"

# Path where bookmarks.json and the filtered views will be written.
output_path = "~/Documents/Claude/Projects/Continual Study and Research/bookmarks.json"
```

Both fields are optional to override — `output_path` has a sensible default if omitted.

## Command-line usage

Once configured, just run the binary:

```bash
raindrop_sync
```

Output:

```
Synced 342 bookmarks to /Users/john/Documents/Claude/Projects/Continual Study and Research/bookmarks.json
Filtered views written to /Users/john/Documents/Claude/Projects/Continual Study and Research:
  last_day_bookmarks.json   — 3 bookmarks
  last_week_bookmarks.json  — 18 bookmarks
  last_month_bookmarks.json — 47 bookmarks
```

### Overriding the API key at runtime

The `RAINDROP_TOKEN` environment variable takes precedence over the config file, useful for one-off runs or CI:

```bash
RAINDROP_TOKEN=your_token raindrop_sync
```

### Install to PATH

```bash
mkdir -p ~/.local/bin

# Option A — symlink (picks up recompiled binaries automatically)
ln -s "$(pwd)/target/release/raindrop_sync" ~/.local/bin/raindrop_sync

# Option B — copy the binary
cp target/release/raindrop_sync ~/.local/bin/raindrop_sync
```

Make sure `~/.local/bin` is on your PATH (add to `~/.zshrc` or `~/.bashrc` if not already):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Shell alias

Add to your `~/.zshrc` or `~/.bashrc` to invoke with a short command:

```bash
alias bsync='raindrop_sync'
```

## Output format

Four files are written to the configured output directory on every sync:

| File | Contents |
|---|---|
| `bookmarks.json` | All bookmarks |
| `last_day_bookmarks.json` | Updated since midnight today |
| `last_week_bookmarks.json` | Updated since Monday 00:00 |
| `last_month_bookmarks.json` | Updated since the 1st of the current month |

Each file is a flat JSON array:

```json
[
  {
    "id": 12345,
    "title": "The Rust Programming Language",
    "link": "https://doc.rust-lang.org/book/",
    "tags": ["rust", "books"],
    "collection_id": 456,
    "collection": "Programming",
    "created": "2024-01-15T10:00:00Z",
    "last_update": "2024-03-01T12:00:00Z",
    "excerpt": "...",
    "type": "link"
  }
]
```

Every run does a full sync and overwrites all files. The sync rate-limits itself to 60 requests/minute (half the API limit of 120/min).

## Scheduling

### macOS — launchd

Create `~/Library/LaunchAgents/com.raindrop_sync.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.raindrop_sync</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Users/john/.local/bin/raindrop_sync</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict>
    <key>Hour</key>
    <integer>3</integer>
    <key>Minute</key>
    <integer>0</integer>
  </dict>
  <key>StandardOutPath</key>
  <string>/tmp/raindrop_sync.log</string>
  <key>StandardErrorPath</key>
  <string>/tmp/raindrop_sync.err</string>
</dict>
</plist>
```

The API key is read from `~/.config/raindrop_sync/config.toml`, so no credentials need to appear in the plist.

Load it:

```bash
launchctl load ~/Library/LaunchAgents/com.raindrop_sync.plist
```

Unload it:

```bash
launchctl unload ~/Library/LaunchAgents/com.raindrop_sync.plist
```

### cron (alternative)

```cron
0 3 * * * ~/.local/bin/raindrop_sync
```

## Development

```bash
cargo build        # compile
cargo test         # run all tests
cargo test <name>  # run a single test by name
cargo clippy       # lint
cargo fmt          # format
```

## License

MIT — see [LICENSE](LICENSE).
