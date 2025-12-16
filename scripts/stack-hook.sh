# stack-game shell hook
# Sends START/END events to /tmp/stack-game.sock for each command.

STACK_SOCK="/tmp/stack-game.sock"
STACK_CMD_ID=0

stack_send() {
  local line="$1"
  if [ -S "$STACK_SOCK" ]; then
    # Try nc -U first; fallback to socat if available.
    if command -v nc >/dev/null 2>&1; then
      printf "%s\n" "$line" | nc -U "$STACK_SOCK" 2>/dev/null || true
    elif command -v socat >/dev/null 2>&1; then
      printf "%s\n" "$line" | socat - UNIX-CONNECT:"$STACK_SOCK" 2>/dev/null || true
    fi
  fi
}

stack_preexec() {
  STACK_CMD_ID=$((STACK_CMD_ID + 1))
  STACK_LAST_CMD="$1"
  stack_send "START ${STACK_CMD_ID} ${STACK_LAST_CMD}"
}

stack_precmd() {
  local code=$?
  if [ -n "$STACK_CMD_ID" ]; then
    stack_send "END ${STACK_CMD_ID} ${code}"
  fi
}

# Zsh integration
if [ -n "$ZSH_VERSION" ]; then
  autoload -Uz add-zsh-hook
  add-zsh-hook preexec stack_preexec
  add-zsh-hook precmd stack_precmd
fi

# Bash integration (uses PROMPT_COMMAND and trap DEBUG)
if [ -n "$BASH_VERSION" ]; then
  trap 'stack_preexec "$BASH_COMMAND"' DEBUG
  PROMPT_COMMAND='stack_precmd'
fi
