use unicode_width::UnicodeWidthChar;

#[derive(Default)]
pub struct CursorX {
    column: usize,
    character: usize,
    byte: usize,
}

impl CursorX {
    pub fn incr_by_char(&mut self, char: char) {
        self.column += char.width().unwrap_or(0);
        self.character += 1;
        self.byte += char.len_utf8();
    }

    pub fn decr_by_char(&mut self, char: char) {
        self.column = self.column.saturating_sub(char.width().unwrap_or(0)).max(0);
        self.character = self.character.saturating_sub(1);
        self.byte = self.byte.saturating_sub(char.len_utf8());
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn character(&self) -> usize {
        self.character
    }

    pub fn byte(&self) -> usize {
        self.byte
    }
}
