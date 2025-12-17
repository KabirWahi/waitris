// Shared game UI/constants.
pub const BOARD_W: usize = 10;
pub const BOARD_H: usize = 20;
pub const CELL_W: usize = 2; // render each block as two characters wide (letter + filler)
pub const PLAY_W: usize = BOARD_W * CELL_W + 2; // inner width plus side walls
pub const PLAY_H: usize = BOARD_H + 2; // inner height plus ceiling/floor
// Minimal pane width to fit the expanded info/controls width + cabinet border.
pub const MIN_PANE_WIDTH: u16 = (PLAY_W as u16) + 2;
pub const CHUNK_SIZE: usize = 8;
pub const SOCKET_PATH: &str = "/tmp/stack-game.sock";
pub const VARIETY_THRESH: i32 = 100;
pub const BOMB_CAP: i32 = 3;
