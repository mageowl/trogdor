#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Pos {
    pub col: usize,
    pub ln: usize,
}

impl Pos {
    pub fn new(ln: usize, col: usize) -> Self {
        Self { ln, col }
    }
    pub fn prev(self) -> Self {
        let Self { ln, col } = self;
        Self {
            ln,
            col: col.saturating_sub(1),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub fn char(pos: Pos) -> Span {
        Self {
            start: pos.prev(),
            end: pos,
        }
    }
}
