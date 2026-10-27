#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct SourcePosition {
    pub start: i32,
    pub finish: i32,
}

impl SourcePosition {
    fn of(start: i32, finish: i32) -> Self {
        Self {
            start,
            finish,
        }
    }
}