# STACK  
*A terminal Tetris game driven by your shell commands*

STACK is a Rust TUI game designed to run alongside a real shell.  
Every command you execute spawns Tetris pieces derived from the command text.  
Long-running commands keep the game busy; failed commands corrupt the board.

This project prioritizes:
- terminal-native UX
- ASCII / old-arcade aesthetics
- indirect control (you generate the game by working)
- zero interference with the real shell

---

## High-level Architecture

- Two panes (recommended via `tmux`)
  - **Left pane**: real shell (zsh/bash)
  - **Right pane**: `stack-game` (Rust TUI)
- Shell sends command lifecycle events to the game
- Game renders and runs independently
- The game never blocks or alters shell execution

---

## Core Concepts

- **Command-driven piece stream**: commands generate pieces
- **Run mode**: while a command executes, pieces continue spawning
- **ASCII arcade style**: monochrome, chunky glyphs, no modern UI
- **Punitive by design**: long or complex commands create more pieces

---

# PHASE 0 — Repo + Tooling

### Goals
- Create a clean Rust workspace
- Establish terminal UI + input loop

### Steps
1. Create Rust binary crate:
   ```sh
   cargo new stack-game
   ```
2. Add dependencies:
   - `ratatui`
   - `crossterm`
   - `rand`
3. Set up:
   - raw terminal mode
   - alternate screen
   - panic-safe terminal restore

---

# PHASE 1 — Base Tetris (no command integration)

## 1.1 Game Board

- Dimensions:
  - Width: 10
  - Height: 20
- Representation:
  ```rust
  enum Cell {
      Empty,
      Filled(char),
  }
  ```
- Board stored as `Vec<Vec<Cell>>`

---

## 1.2 Pieces

### Supported Shapes
- Tetrominoes: `I O T S Z J L`
- No hold piece
- Rotations: 4 states per piece

### Piece Struct
```rust
struct Piece {
    shape: Shape,
    rotation: u8,
    x: i32,
    y: i32,
    payload: Vec<char>, // length 1–4
}
```

---

## 1.3 Controls

| Key       | Action        |
|----------|---------------|
| ← / →    | Move left/right |
| ↓        | Soft drop     |
| ↑        | Rotate        |
| Space    | Hard drop     |

---

## 1.4 Gravity + Locking

- Gravity tick (start ~450ms)
- If collision on gravity:
  - lock piece
  - write payload chars to board
  - clear lines
  - spawn next piece

---

## 1.5 Line Clears + Scoring

Classic scoring only (initially):

| Lines | Score |
|------|-------|
| 1    | 100   |
| 2    | 300   |
| 3    | 500   |
| 4    | 800   |

---

## 1.6 Visual Style (ASCII / Soviet Arcade)

- No colors or very limited (white / gray)
- Borders using `+ - |` or box-drawing
- Cells:
  - Empty: ` `
  - Filled: payload char (or fallback `█`)
- Text labels:
  - `SCORE`
  - `LINES`
  - `MODE: IDLE / RUN`
- Game over message:
  ```
  ПРОВАЛ
  ```

---

# PHASE 2 — Command-driven Pieces (no shell capture yet)

## 2.1 Tokenization v1 (simple)

- Split command string by whitespace:
  ```text
  git commit -m fix
  → ["git", "commit", "-m", "fix"]
  ```
- Keep all visible ASCII characters
- Ignore empty tokens

---

## 2.2 Chunking Rules

For **each token independently**:
- Greedily chunk characters into:
  - size 4
  - remainder 1–3

Example:
```
"commit" → ["comm", "it"]
```

---

## 2.3 Shape Mapping by Chunk Size

| Chunk size | Shape type |
|-----------|------------|
| 4         | Tetromino  |
| 3         | Triomino   |
| 2         | Domino     |
| 1         | Monomino   |

Each chunk becomes **one piece**.

---

## 2.4 Payload Assignment

- Each cell of a piece renders one char from the chunk
- Order is deterministic (top-left → bottom-right)
- If shape has fewer cells than chars, truncate
- If more cells, repeat last char

---

## 2.5 CommandRun Model

```rust
struct CommandRun {
    id: u64,
    tokens: Vec<String>,
    current_token: usize,
    current_chunk: usize,
    cycle: usize,
    active: bool,
}
```

---

## 2.6 Repeating Cycles (Long-running commands)

- First cycle:
  - consume all chunks derived from the command
- If command still running:
  - reset chunk index
  - increment `cycle`
  - repeat chunks again
  - shapes are randomized per cycle

This continues **until the command finishes**.

---

# PHASE 3 — Shell Integration

## 3.1 Transport

- Game listens on a Unix socket:
  ```
  /tmp/stack-game.sock
  ```

---

## 3.2 Events

### COMMAND_START
```
START <id> <command string>
```

### COMMAND_END
```
END <id> <exit_code>
```

---

## 3.3 Shell Hook (zsh/bash)

Responsibilities:
- On Enter:
  - send START event
- After command completes:
  - send END event with `$?`

Game does **not** execute commands.

---

# PHASE 4 — Run Mode & Timing

## 4.1 Modes

- **IDLE**
  - normal gravity
  - no new command cycles
- **RUN**
  - gravity increased
  - command chunks stream continues

Minimum RUN duration:
- Even instant commands produce at least one piece

---

# PHASE 5 — Success / Failure Effects

## 5.1 Success (exit code 0)

Grant **one Success Bomb** (stackable, capped).

### Success Bomb Behavior
- Special piece appears in queue
- On lock:
  - clears a 3×3 area centered on drop
- Uses normal gravity and controls

---

## 5.2 Failure (exit code ≠ 0)

Apply both:
1. **Garbage row**
   - added at bottom
   - single random hole
2. **Infection**
   - select N random filled cells
   - replace their character with `?`
   - cosmetic only

---

# PHASE 6 — Variety Scoring

## 6.1 Command Identity

- Identity = first token
  ```
  git commit → "git"
  cargo test → "cargo"
  ```

---

## 6.2 Variety Bonus

- Track last command identity
- If different:
  - `+VARIETY_SCORE` (e.g. +25)
- If same:
  - no bonus

Added directly to score (not a multiplier).

---

# PHASE 7 — Quote-aware Tokenization (optional later)

## Goal
Support:
```
gt create -all "first commit"
→ gt, create, -all, first, commit
```

## Rules
- Quotes removed
- Quoted content split on whitespace
- No escape handling initially
- Unmatched quote consumes rest of string

This replaces `split_whitespace()`.

---

# Design Philosophy

- You generate the chaos you must survive
- Long commands = more pressure
- The game never blocks your work
- ASCII over polish
- Punishment is allowed (and intentional)

---

# Non-goals

- Full shell parsing
- Multiplayer
- Mouse support
- Fancy colors
- Undo / rewind

---

# MVP Completion Criteria

- Tetris playable
- Commands spawn letter-based pieces
- Long-running commands repeat cycles
- Success bomb + failure corruption
- Variety scoring active
- Stable for long sessions

---

# Possible Names (CLI-style)

- `stack`
- `heap`
- `overflow`
- `fall`
- `blocks`

---

END
