use raylib::ffi::Vector2;

pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<Position> for Vector2 {
    fn from(val: Position) -> Self {
        Vector2 {
            x: val.x as f32,
            y: val.y as f32,
        }
    }
}
