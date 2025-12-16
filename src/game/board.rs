#[derive(Clone, Copy)]
pub enum Cell {
    Empty,
    Filled(char, char),
}

#[derive(Clone)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::Empty; width * height],
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.idx(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, value: Cell) {
        let idx = self.idx(x, y);
        self.cells[idx] = value;
    }
}
