use std::fmt;

/// A position on the game grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub col: i32,
    pub row: i32,
}

impl GridPos {
    pub const fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    /// Apply a direction to get a new position.
    pub fn step(self, dir: Direction) -> Self {
        let (dc, dr) = dir.delta();
        Self {
            col: self.col + dc,
            row: self.row + dr,
        }
    }
}

impl fmt::Display for GridPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.col, self.row)
    }
}

/// Which side of the river.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bank {
    Left,
    Right,
}

impl Bank {
    pub fn opposite(self) -> Bank {
        match self {
            Bank::Left => Bank::Right,
            Bank::Right => Bank::Left,
        }
    }
}

/// Movement directions on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,    // row - 1
    Down,  // row + 1
    Left,  // col - 1
    Right, // col + 1
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

// --- Grid layout constants ---

pub const GRID_COLS: i32 = 12;
pub const GRID_ROWS: i32 = 8;

pub const LEFT_BANK_COL_MIN: i32 = 0;
pub const LEFT_BANK_COL_MAX: i32 = 3;
pub const RIVER_COL_MIN: i32 = 4;
pub const RIVER_COL_MAX: i32 = 7;
pub const RIGHT_BANK_COL_MIN: i32 = 8;
pub const RIGHT_BANK_COL_MAX: i32 = 11;

pub const DOCK_ROW: i32 = 4;
pub const LEFT_DOCK: GridPos = GridPos::new(3, DOCK_ROW);
pub const RIGHT_DOCK: GridPos = GridPos::new(8, DOCK_ROW);

// Starting positions
pub const PLAYER_START: GridPos = GridPos::new(2, 4);
pub const WOLF_START: GridPos = GridPos::new(1, 2);
pub const SHEEP_START: GridPos = GridPos::new(1, 4);
pub const CABBAGE_START: GridPos = GridPos::new(1, 6);

// --- Isometric rendering constants ---

/// Tile dimensions in world units (the virtual coordinate space).
pub const TILE_WIDTH: f32 = 64.0;
pub const TILE_HEIGHT: f32 = 22.0;

/// The virtual world dimensions that the camera maps to screen.
pub const WORLD_HEIGHT: f32 = 500.0;

/// Check if a grid position is walkable land.
pub fn is_walkable(pos: GridPos) -> bool {
    pos.row >= 0
        && pos.row < GRID_ROWS
        && pos.col >= 0
        && pos.col < GRID_COLS
        && !(pos.col >= RIVER_COL_MIN && pos.col <= RIVER_COL_MAX)
}

/// Determine which bank a position is on, if any.
pub fn bank_of(pos: GridPos) -> Option<Bank> {
    if pos.col >= LEFT_BANK_COL_MIN && pos.col <= LEFT_BANK_COL_MAX {
        Some(Bank::Left)
    } else if pos.col >= RIGHT_BANK_COL_MIN && pos.col <= RIGHT_BANK_COL_MAX {
        Some(Bank::Right)
    } else {
        None
    }
}

/// Check if two positions are adjacent (Manhattan distance <= 1).
pub fn is_adjacent(a: GridPos, b: GridPos) -> bool {
    (a.col - b.col).abs() + (a.row - b.row).abs() <= 1
}

/// Check if a position is the dock for the given bank.
pub fn is_dock_position(pos: GridPos, bank: Bank) -> bool {
    match bank {
        Bank::Left => pos == LEFT_DOCK,
        Bank::Right => pos == RIGHT_DOCK,
    }
}

/// Get the dock position for a bank.
pub fn dock_for(bank: Bank) -> GridPos {
    match bank {
        Bank::Left => LEFT_DOCK,
        Bank::Right => RIGHT_DOCK,
    }
}

/// Convert grid (col, row) to isometric world coordinates.
/// Returns the center of the tile's top diamond face.
pub fn grid_to_iso(pos: GridPos) -> (f32, f32) {
    // Center the grid horizontally in the world.
    // Total iso width = (GRID_COLS + GRID_ROWS) * TILE_WIDTH / 2 = 20 * 32 = 640
    // Total iso height = (GRID_COLS + GRID_ROWS) * TILE_HEIGHT / 2 = 20 * 11 = 220
    // We want this centered with padding for sprites above tiles and HUD below.
    let x_origin = 440.0; // roughly center for 16:9 aspect
    let y_origin = 100.0;

    let iso_x = x_origin + (pos.col as f32 - pos.row as f32) * (TILE_WIDTH / 2.0);
    let iso_y = y_origin + (pos.col as f32 + pos.row as f32) * (TILE_HEIGHT / 2.0);

    (iso_x, iso_y)
}
