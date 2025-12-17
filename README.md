# WAITRIS

WAITRIS is a terminal Tetris game driven by your shell commands. It launches a tmux split: your shell on the left, the game on the right.

## Prerequisites

- Rust toolchain (for building from source)
- tmux
- socat

Install tmux + socat:

```sh
# macOS (Homebrew)
brew install tmux socat

# Debian/Ubuntu
sudo apt-get install tmux socat
```

## Install (from source)

From the repo root:

```sh
cargo install --path . --bin waitris
```

Make sure your cargo bin directory is on PATH:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

## Run

```sh
waitris
```

To quit the whole session from the left pane:

```sh
waitris quit
```

## Shell Hook (required)

The hook streams START/END events for each shell command to the game.

Install:

```sh
waitris install-hook
```

Uninstall:

```sh
waitris uninstall-hook
```

## Notes

- The game listens on `/tmp/stack-game.sock`.

## One‑line installer (placeholder)

```sh
curl -fsSL https://example.com/waitris/install.sh | sh
```

This will install `waitris` and run `waitris install-hook` for you.
