// Position and Span for source code locations

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }

    pub fn single(pos: Position) -> Self {
        Span {
            start: pos.clone(),
            end: Position::new(pos.line, pos.column + 1),
        }
    }
}