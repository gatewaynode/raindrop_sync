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

### 3. Configure the output path (optional)

Edit `config.toml` in the project root:

```toml
output_path = "~/Documents/Claude/Projects/Continual Study and Research/bookmarks.json"
```

If `config.toml` is absent, this default path is used. The `~` is expanded to `$HOME` at runtime.

## Command-line usage

Set your token and run:

```bash
export RAINDROP_TOKEN=your_token_here
./target/release/raindrop_sync
```

Or inline for a one-off sync:

```bash
RAINDROP_TOKEN=your_token_here ./target/release/raindrop_sync
```

Output:

```
Synced 342 bookmarks to /Users/john/Documents/Claude/Projects/Continual Study and Research/bookmarks.json
```

### Install to PATH

To run `raindrop_sync` from anywhere:

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

Then from any directory:

```bash
RAINDROP_TOKEN=your_token raindrop_sync
```

### Shell alias

Add to your `~/.zshrc` or `~/.bashrc` to set the token once and run with a short command:

```bash
export RAINDROP_TOKEN=your_token_here
alias bsync='raindrop_sync'
```

Then just:

```bash
bsync
```

## Output format

The output file is a flat JSON array — one object per bookmark:

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

Every run does a full sync and overwrites the file. The sync rate-limits itself to 60 requests/minute (half the API limit of 120/min), so it is safe to run frequently without risk of being throttled.

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
    <string>~/.local/bin/raindrop_sync</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>RAINDROP_TOKEN</key>
    <string>your_token_here</string>
  </dict>
  <key>WorkingDirectory</key>
  <string>/path/to/raindrop_sync</string>
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
0 3 * * * RAINDROP_TOKEN=your_token ~/.local/bin/raindrop_sync
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
