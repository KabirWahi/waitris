use std::collections::{HashMap, VecDeque};

use crate::commands;
use crate::game::{random_shape, Board, Cell, Piece, Shape};
use crate::{BOARD_H, BOARD_W, CHUNK_SIZE};


#[derive(Debug)]
pub enum CommandEvent {
    Start { id: u64, command: String },
    End { id: u64, _exit_code: i32 },
}

pub(crate) struct QueuedPiece {
    pub run_id: u64,
    pub cycle: u64,
    pub piece: Piece,
    pub is_bomb: bool,
}

pub(crate) struct CommandRun {
    pub id: u64,
    pub chunks: Vec<String>,
    pub cycle: u64,
    pub active: bool,
    pub identity: String,
}

impl CommandRun {
    fn new(id: u64, chunks: Vec<String>, identity: String) -> Self {
        Self {
            id,
            chunks,
            cycle: 0,
            active: true,
            identity,
        }
    }

    fn next_cycle_pieces(&mut self) -> (u64, Vec<Piece>) {
        self.cycle = self.cycle.wrapping_add(1);
        let mut pieces = Vec::new();
        for chunk in &self.chunks {
            let payload = commands::chunk_to_payload(chunk);
            let shape = random_shape();
            pieces.push(Piece::with_payload(shape, payload));
        }
        (self.cycle, pieces)
    }
}

pub struct Game {
    pub board: Board,
    pub current: Piece,
    pub game_over: bool,
    pub score: u64,
    pub lines_cleared: u64,
    pub pending_clear: Vec<usize>,
    pub clear_flash_frames: u8,
    pub lock_flash_cells: Vec<(usize, usize)>,
    pub lock_flash_frames: u8,
    piece_queue: VecDeque<QueuedPiece>,
    pub active_piece: bool,
    active_run: Option<u64>,
    active_runs: HashMap<u64, CommandRun>,
    pub bombs: i32,
    pub current_is_bomb: bool,
    pub variety_meter: i32,
    pub last_cmd_identity: Option<String>,
    pub variety_streak: i32,
}

impl Game {
    pub fn new() -> Self {
        let board = Board::new(BOARD_W, BOARD_H);
        let game = Self {
            board,
            current: Piece::with_payload(Shape::I, vec!['░'; CHUNK_SIZE]),
            game_over: false,
            score: 0,
            lines_cleared: 0,
            pending_clear: Vec::new(),
            clear_flash_frames: 0,
            lock_flash_cells: Vec::new(),
            lock_flash_frames: 0,
            piece_queue: VecDeque::new(),
            active_piece: false,
            active_run: None,
            active_runs: HashMap::new(),
            bombs: 0,
            current_is_bomb: false,
            variety_meter: 0,
            last_cmd_identity: None,
            variety_streak: 0,
        };
        game
    }

    pub fn can_place(&self, piece: &Piece) -> bool {
        for (x, y, _) in piece.cells() {
            if x < 0 || y < 0 {
                return false;
            }
            let (xu, yu) = (x as usize, y as usize);
            if xu >= self.board.width || yu >= self.board.height {
                return false;
            }
            if let Cell::Filled(_, _) = self.board.get(xu, yu) {
                return false;
            }
        }
        true
    }

    pub fn lock_piece(&mut self) {
        self.lock_flash_cells.clear();
        for (x, y, (left, right)) in self.current.cells_with_pairs() {
            if x >= 0 && y >= 0 {
                let (xu, yu) = (x as usize, y as usize);
                if xu < self.board.width && yu < self.board.height {
                    self.board.set(xu, yu, Cell::Filled(left, right));
                    self.lock_flash_cells.push((xu, yu));
                }
            }
        }
        self.lock_flash_frames = 1;
        self.active_run = None;
        self.active_piece = false;
        let full_rows: Vec<usize> = (0..self.board.height)
            .filter(|y| (0..self.board.width).all(|x| matches!(self.board.get(x, *y), Cell::Filled(_, _))))
            .collect();
        if !full_rows.is_empty() {
            self.pending_clear = full_rows;
            self.clear_flash_frames = 2;
        }

        if self.current_is_bomb {
            self.apply_bomb_clear();
        }
    }

    pub fn move_current(&mut self, dx: i32, dy: i32) -> bool {
        if self.game_over {
            return false;
        }
        let next = self.current.shifted(dx, dy);
        if self.can_place(&next) {
            self.current = next;
            true
        } else {
            false
        }
    }

    pub fn rotate_current(&mut self) -> bool {
        if self.game_over {
            return false;
        }
        let next = self.current.rotated();
        if self.can_place(&next) {
            self.current = next;
            true
        } else {
            false
        }
    }

    pub fn tick_gravity(&mut self) {
        if self.game_over {
            return;
        }
        if !self.active_piece {
            return;
        }
        if !self.move_current(0, 1) {
            self.lock_piece();
            self.spawn_next();
        }
    }

    pub fn hard_drop(&mut self) {
        if self.game_over {
            return;
        }
        if !self.active_piece {
            return;
        }
        while self.move_current(0, 1) {}
        self.lock_piece();
        self.spawn_next();
    }

    pub fn process_effects(&mut self) {
        if self.lock_flash_frames > 0 {
            self.lock_flash_frames -= 1;
        }
        if self.clear_flash_frames > 0 {
            self.clear_flash_frames -= 1;
            if self.clear_flash_frames == 0 && !self.pending_clear.is_empty() {
                self.perform_pending_clear();
            }
        }
    }

    pub fn spawn_next(&mut self) {
        self.ensure_queue();
        if let Some(qp) = self.piece_queue.pop_front() {
            self.active_piece = true;
            self.active_run = if qp.is_bomb { None } else { Some(qp.run_id) };
            self.current_is_bomb = qp.is_bomb;
            if self.can_place(&qp.piece) {
                self.current = qp.piece;
            } else {
                self.game_over = true;
            }
        } else {
            self.active_piece = false;
            self.active_run = None;
            self.current_is_bomb = false;
        }
    }

    pub(crate) fn ghost_piece(&self) -> Piece {
        let mut ghost = self.current.clone();
        while {
            let next = ghost.shifted(0, 1);
            self.can_place(&next)
        } {
            ghost.y += 1;
        }
        ghost
    }

    pub fn handle_command_event(&mut self, ev: CommandEvent) {
        match ev {
            CommandEvent::Start { id, command } => {
                let chunks = commands::command_to_chunks(&command);
                let identity = command_identity(&command);
                let mut run = CommandRun::new(id, chunks, identity.clone());
                let (cycle, pieces) = run.next_cycle_pieces();
                for p in pieces {
                    self.piece_queue.push_back(QueuedPiece {
                        run_id: id,
                        cycle,
                        piece: p,
                        is_bomb: false,
                    });
                }
                self.active_runs.insert(id, run);
                self.last_cmd_identity.get_or_insert(identity);
                if !self.active_piece {
                    self.spawn_next();
                }
            }
            CommandEvent::End { id, _exit_code } => {
                let identity = self.active_runs.get(&id).map(|r| r.identity.clone());
                if let Some(run) = self.active_runs.get_mut(&id) {
                    run.active = false;
                }
                // Drop queued pieces from repeat cycles for this run.
                self.piece_queue
                    .retain(|qp| qp.run_id != id || qp.cycle <= 1);

                if _exit_code != 0 {
                    self.apply_garbage_row();
                    self.apply_infection();
                }
                if let Some(id_str) = identity {
                    self.apply_variety(&id_str, _exit_code);
                    self.last_cmd_identity = Some(id_str);
                }
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.active_piece
            || !self.piece_queue.is_empty()
            || self.active_runs.values().any(|r| r.active)
    }

    fn ensure_queue(&mut self) {
        if !self.piece_queue.is_empty() {
            return;
        }
        for run in self.active_runs.values_mut() {
            if run.active {
                let (cycle, pieces) = run.next_cycle_pieces();
                for p in pieces {
                    self.piece_queue.push_back(QueuedPiece {
                        run_id: run.id,
                        cycle,
                        piece: p,
                        is_bomb: false,
                    });
                }
            }
        }
        if self.piece_queue.is_empty() && self.bombs > 0 {
            let bomb = Self::make_bomb_piece();
            self.piece_queue.push_back(QueuedPiece {
                run_id: 0,
                cycle: 0,
                piece: bomb,
                is_bomb: true,
            });
            self.bombs -= 1;
        }
    }

    fn add_score(&mut self, cleared: u64) {
        let add = match cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
        self.score += add;
    }

    fn make_bomb_piece() -> Piece {
        // Use O piece for compact 2x2 bomb footprint with solid payload.
        Piece::with_payload(Shape::O, vec!['▓'; CHUNK_SIZE])
    }

    fn perform_pending_clear(&mut self) {
        let cleared = self.pending_clear.len() as u64;
        if cleared == 0 {
            return;
        }
        let mut new_cells = Vec::with_capacity(self.board.cells.len());
        for y in 0..self.board.height {
            if self.pending_clear.contains(&y) {
                continue;
            }
            for x in 0..self.board.width {
                new_cells.push(self.board.get(x, y));
            }
        }
        for _ in 0..cleared {
            for _ in 0..self.board.width {
                new_cells.insert(0, Cell::Empty);
            }
        }
        self.board.cells = new_cells;
        self.lines_cleared += cleared;
        self.add_score(cleared);
        self.pending_clear.clear();
    }
}

fn command_identity(cmd: &str) -> String {
    commands::tokenize_command(cmd)
        .into_iter()
        .next()
        .unwrap_or_default()
}
