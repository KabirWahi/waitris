use std::error::Error;

mod app;
mod config;
mod game;
mod io;
mod ui;
mod commands;
pub use game::{CommandEvent, Game};
pub use config::{
    BOMB_CAP, BOARD_H, BOARD_W, CELL_W, CHUNK_SIZE, MIN_PANE_WIDTH, PLAY_H, PLAY_W, SOCKET_PATH,
    VARIETY_THRESH,
};

fn main() -> Result<(), Box<dyn Error>> {
    app::run()
}
