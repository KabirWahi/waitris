use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone, Copy)]
pub enum Shape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[derive(Clone)]
pub struct Piece {
    pub shape: Shape,
    pub rotation: u8,
    pub x: i32,
    pub y: i32,
    pub payload: Vec<char>,
}

impl Piece {
    pub fn with_payload(shape: Shape, payload: Vec<char>) -> Self {
        Self {
            shape,
            rotation: 0,
            x: 3,
            y: 0,
            payload,
        }
    }

    pub fn cells(&self) -> Vec<(i32, i32, char)> {
        let offsets = shape_offsets(self.shape, self.rotation);
        offsets
            .iter()
            .enumerate()
            .map(|(i, (dx, dy))| {
                let ch = self.payload.get(i).copied().unwrap_or_else(|| {
                    *self.payload.last().unwrap_or(&'#')
                });
                (self.x + dx, self.y + dy, ch)
            })
            .collect()
    }

    pub fn cells_with_pairs(&self) -> Vec<(i32, i32, (char, char))> {
        let offsets = shape_offsets(self.shape, self.rotation);
        offsets
            .iter()
            .enumerate()
            .map(|(i, (dx, dy))| {
                let left = *self.payload.get(i * 2).or(self.payload.last()).unwrap_or(&'░');
                let right = *self
                    .payload
                    .get(i * 2 + 1)
                    .or(self.payload.last())
                    .unwrap_or(&'░');
                (self.x + dx, self.y + dy, (left, right))
            })
            .collect()
    }

    pub fn rotated(&self) -> Self {
        let mut next = self.clone();
        next.rotation = (next.rotation + 1) % 4;
        next
    }

    pub fn shifted(&self, dx: i32, dy: i32) -> Self {
        let mut next = self.clone();
        next.x += dx;
        next.y += dy;
        next
    }
}

pub fn random_shape() -> Shape {
    let shapes = [
        Shape::I,
        Shape::O,
        Shape::T,
        Shape::S,
        Shape::Z,
        Shape::J,
        Shape::L,
    ];
    let mut rng = thread_rng();
    *shapes.choose(&mut rng).unwrap_or(&Shape::I)
}

pub fn shape_offsets(shape: Shape, rotation: u8) -> &'static [(i32, i32)] {
    const I: [[(i32, i32); 4]; 4] = [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
    ];
    const O: [[(i32, i32); 4]; 4] = [
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
    ];
    const T: [[(i32, i32); 4]; 4] = [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ];
    const S: [[(i32, i32); 4]; 4] = [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ];
    const Z: [[(i32, i32); 4]; 4] = [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
    ];
    const J: [[(i32, i32); 4]; 4] = [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ];
    const L: [[(i32, i32); 4]; 4] = [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ];

    let r = (rotation % 4) as usize;
    match shape {
        Shape::I => &I[r],
        Shape::O => &O[r],
        Shape::T => &T[r],
        Shape::S => &S[r],
        Shape::Z => &Z[r],
        Shape::J => &J[r],
        Shape::L => &L[r],
    }
}
