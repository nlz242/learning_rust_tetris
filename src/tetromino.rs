use rand::Rng;

// Represents the 7 standard Tetris shapes
#[derive(Clone, Copy, Debug)] 
pub enum TetrominoShape {
    I, O, T, S, Z, J, L
}

// A simple coordinate type
pub type Point = (i32, i32);

impl TetrominoShape {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        TetrominoShape::from_index(rng.random_range(0..7))
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => TetrominoShape::I,
            1 => TetrominoShape::O,
            2 => TetrominoShape::T,
            3 => TetrominoShape::S,
            4 => TetrominoShape::Z,
            5 => TetrominoShape::J,
            _ => TetrominoShape::L,
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            TetrominoShape::I => 0,
            TetrominoShape::O => 1,
            TetrominoShape::T => 2,
            TetrominoShape::S => 3,
            TetrominoShape::Z => 4,
            TetrominoShape::J => 5,
            TetrominoShape::L => 6,
        }
    }

    // Returns the 4 coordinates that make up this shape.
    // The coordinates are relative to a pivot point (0,0).
    // We return a fixed-size array of 4 Points.
    pub fn cells(&self) -> [Point; 4] {
        match self {
            TetrominoShape::I => [(0, 0), (-1, 0), (1, 0), (2, 0)],
            TetrominoShape::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            TetrominoShape::T => [(0, 0), (-1, 0), (1, 0), (0, 1)],
            TetrominoShape::S => [(0, 0), (-1, 0), (0, 1), (1, 1)],
            TetrominoShape::Z => [(0, 0), (1, 0), (0, 1), (-1, 1)],
            TetrominoShape::J => [(0, 0), (-1, 0), (1, 0), (-1, 1)],
            TetrominoShape::L => [(0, 0), (-1, 0), (1, 0), (1, 1)],
        }
    }
}
