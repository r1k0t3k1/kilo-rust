use std::ops;

pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }
}

impl ops::Add<Position> for Position {
    type Output = Position;
fn add(self, rhs: Position) -> Position {
        let ret_x = self.x.saturating_add(rhs.x);
        let ret_y = self.y.saturating_add(rhs.y);

        Position { x: ret_x, y: ret_y }
    }
}

impl ops::Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Position {
        let ret_x = self.x.saturating_sub(rhs.x);
        let ret_y = self.y.saturating_sub(rhs.y);

        Position { x: ret_x, y: ret_y }
    }
}

impl ops::AddAssign<Position> for Position {
    fn add_assign(&mut self, rhs: Position) {
        *self = Self {
            x: self.x.saturating_add(rhs.x),
            y: self.y.saturating_add(rhs.y),
        }
    }
}

impl ops::SubAssign<Position> for Position {
    fn sub_assign(&mut self, rhs: Position) {
        *self = Self {
            x: self.x.saturating_sub(rhs.x),
            y: self.y.saturating_sub(rhs.y),
        }
    }
}

