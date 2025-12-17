use rand::seq::IteratorRandom;
use rand::Rng;

use crate::game::Cell;
use crate::{BOMB_CAP, VARIETY_THRESH};

use super::Game;

impl Game {
    pub(super) fn apply_bomb_clear(&mut self) {
        let mut to_clear = Vec::new();
        for (x, y, _) in self.current.cells() {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && ny >= 0 {
                        let (xu, yu) = (nx as usize, ny as usize);
                        if xu < self.board.width && yu < self.board.height {
                            to_clear.push((xu, yu));
                        }
                    }
                }
            }
        }
        to_clear.sort();
        to_clear.dedup();
        for (x, y) in to_clear {
            self.board.set(x, y, Cell::Empty);
        }
    }

    pub(super) fn apply_garbage_row(&mut self) {
        let mut rng = rand::thread_rng();
        let hole = rng.gen_range(0..self.board.width);
        let mut new_cells = vec![Cell::Empty; self.board.width * self.board.height];
        // shift everything up by one row
        for y in 1..self.board.height {
            for x in 0..self.board.width {
                let src = self.board.get(x, y);
                let dst_idx = (y - 1) * self.board.width + x;
                new_cells[dst_idx] = src;
            }
        }
        // bottom row with garbage except hole
        for x in 0..self.board.width {
            let idx = (self.board.height - 1) * self.board.width + x;
            if x == hole {
                new_cells[idx] = Cell::Empty;
            } else {
                new_cells[idx] = Cell::Filled('#', '░');
            }
        }
        // If top row had filled cells, game over.
        let overflow = (0..self.board.width).any(|x| matches!(self.board.get(x, 0), Cell::Filled(_, _)));
        self.board.cells = new_cells;
        if overflow {
            self.game_over = true;
        }
    }

    pub(super) fn apply_infection(&mut self) {
        let mut rng = rand::thread_rng();
        let mut filled: Vec<(usize, usize)> = Vec::new();
        for y in 0..self.board.height {
            for x in 0..self.board.width {
                if let Cell::Filled(_, _) = self.board.get(x, y) {
                    filled.push((x, y));
                }
            }
        }
        let count = filled.len().min(5);
        for &(x, y) in filled.iter().choose_multiple(&mut rng, count) {
            self.board.set(x, y, Cell::Filled('?', '░'));
        }
    }

    pub(super) fn apply_variety(&mut self, identity: &str, exit_code: i32) {
        let same_as_last = self.last_cmd_identity.as_deref() == Some(identity);
        if same_as_last {
            self.variety_meter = (self.variety_meter - 5).max(0);
            self.variety_streak = 0;
        } else {
            self.variety_meter = (self.variety_meter - 2).max(0);
            self.variety_streak += 1;
        }

        let mut variety_points = if same_as_last {
            0
        } else {
            10 + 3 * (self.variety_streak.min(10))
        };

        if exit_code != 0 {
            variety_points /= 2;
        }

        self.variety_meter += variety_points;

        while self.variety_meter >= VARIETY_THRESH {
            self.variety_meter -= VARIETY_THRESH;
            self.bombs = (self.bombs + 1).min(BOMB_CAP);
        }
    }
}
