use crate::Formatter;

pub struct App {
    pub lines: Vec<String>,
    pub offset: usize,
    pub columns: usize,
    pub formatter: Formatter,
}

impl App {
    pub fn new(lines: Vec<String>, columns: usize, formatter: Formatter) -> Self {
        Self {
            lines,
            offset: 0,
            columns,
            formatter,
        }
    }

    pub fn replace_content(&mut self, lines: Vec<String>, formatter: Formatter) {
        self.lines = lines;
        self.formatter = formatter;
        self.offset = 0;
    }

    pub fn scroll_down(&mut self, page_height: usize) {
        let max_offset = self.max_offset(page_height);
        self.offset = (self.offset + 1).min(max_offset);
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    pub fn page_down(&mut self, page_height: usize) {
        let max_offset = self.max_offset(page_height);
        self.offset = (self.offset + page_height).min(max_offset);
    }

    pub fn page_up(&mut self, page_height: usize) {
        self.offset = self.offset.saturating_sub(page_height);
    }

    pub fn scroll_to_end(&mut self, page_height: usize) {
        self.offset = self.max_offset(page_height);
    }

    // The last valid offset keeps at least one line visible in the first column.
    fn max_offset(&self, page_height: usize) -> usize {
        let total_rows = self.columns * page_height;
        self.lines.len().saturating_sub(total_rows)
    }

    /// Returns the slice of lines that column `col` should render.
    pub fn column_lines(&self, col: usize, page_height: usize) -> &[String] {
        let start = self.offset + col * page_height;
        let end = (start + page_height).min(self.lines.len());
        if start >= self.lines.len() {
            &[]
        } else {
            &self.lines[start..end]
        }
    }
}
