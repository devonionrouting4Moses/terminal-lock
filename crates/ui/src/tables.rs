use colored::*;
use crate::colors::ColorPalette;

/// Beautiful rounded corner table renderer
pub struct Table {
    palette: ColorPalette,
    title: Option<String>,
    rows: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl Table {
    pub fn new(palette: ColorPalette) -> Self {
        Self {
            palette,
            title: None,
            rows: Vec::new(),
            headers: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_headers(mut self, headers: Vec<&str>) -> Self {
        self.headers = headers.iter().map(|h| h.to_string()).collect();
        self
    }

    pub fn add_row(mut self, row: Vec<&str>) -> Self {
        self.rows.push(row.iter().map(|r| r.to_string()).collect());
        self
    }

    /// Render the table with rounded corners
    pub fn render(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let mut output = String::new();

        // Calculate column widths
        let col_widths = self.calculate_column_widths();

        // Top border with rounded corners
        output.push_str(&self.render_top_border(&col_widths));
        output.push('\n');

        // Title if present
        if let Some(title) = &self.title {
            output.push_str(&self.render_title_row(title, &col_widths));
            output.push('\n');
            output.push_str(&self.render_separator(&col_widths));
            output.push('\n');
        }

        // Headers
        if !self.headers.is_empty() {
            output.push_str(&self.render_row(&self.headers, &col_widths, true));
            output.push('\n');
            output.push_str(&self.render_separator(&col_widths));
            output.push('\n');
        }

        // Data rows
        for (idx, row) in self.rows.iter().enumerate() {
            output.push_str(&self.render_row(row, &col_widths, false));
            output.push('\n');

            // Add separator between rows for visual appeal
            if idx < self.rows.len() - 1 {
                output.push_str(&self.render_light_separator(&col_widths));
                output.push('\n');
            }
        }

        // Bottom border with rounded corners
        output.push_str(&self.render_bottom_border(&col_widths));

        output
    }

    fn calculate_column_widths(&self) -> Vec<usize> {
        let mut widths = vec![0; self.headers.len().max(1)];

        // Check header widths
        for (i, header) in self.headers.iter().enumerate() {
            widths[i] = widths[i].max(header.len());
        }

        // Check row widths
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Add padding
        widths.iter_mut().for_each(|w| *w += 2);
        widths
    }

    fn render_top_border(&self, col_widths: &[usize]) -> String {
        let mut border = String::new();
        border.push_str(&"╭".bright_cyan().bold().to_string());

        for (i, &width) in col_widths.iter().enumerate() {
            border.push_str(&"─".repeat(width).bright_cyan().bold().to_string());
            if i < col_widths.len() - 1 {
                border.push_str(&"┬".bright_cyan().bold().to_string());
            }
        }

        border.push_str(&"╮".bright_cyan().bold().to_string());
        border
    }

    fn render_bottom_border(&self, col_widths: &[usize]) -> String {
        let mut border = String::new();
        border.push_str(&"╰".bright_cyan().bold().to_string());

        for (i, &width) in col_widths.iter().enumerate() {
            border.push_str(&"─".repeat(width).bright_cyan().bold().to_string());
            if i < col_widths.len() - 1 {
                border.push_str(&"┴".bright_cyan().bold().to_string());
            }
        }

        border.push_str(&"╯".bright_cyan().bold().to_string());
        border
    }

    fn render_separator(&self, col_widths: &[usize]) -> String {
        let mut sep = String::new();
        sep.push_str(&"├".bright_cyan().to_string());

        for (i, &width) in col_widths.iter().enumerate() {
            sep.push_str(&"─".repeat(width).bright_cyan().to_string());
            if i < col_widths.len() - 1 {
                sep.push_str(&"┼".bright_cyan().to_string());
            }
        }

        sep.push_str(&"┤".bright_cyan().to_string());
        sep
    }

    fn render_light_separator(&self, col_widths: &[usize]) -> String {
        let mut sep = String::new();
        sep.push_str(&"│".bright_black().to_string());

        for (i, &width) in col_widths.iter().enumerate() {
            sep.push_str(&" ".repeat(width));
            if i < col_widths.len() - 1 {
                sep.push_str(&"│".bright_black().to_string());
            }
        }

        sep.push_str(&"│".bright_black().to_string());
        sep
    }

    fn render_title_row(&self, title: &str, col_widths: &[usize]) -> String {
        let total_width: usize = col_widths.iter().sum::<usize>() + col_widths.len() + 1;
        let padding = (total_width.saturating_sub(title.len())) / 2;

        let mut row = String::new();
        row.push_str(&"│".bright_cyan().bold().to_string());
        row.push_str(&" ".repeat(padding));
        row.push_str(&title.bright_yellow().bold().to_string());
        row.push_str(&" ".repeat(total_width.saturating_sub(title.len()).saturating_sub(padding).saturating_sub(2)));
        row.push_str(&"│".bright_cyan().bold().to_string());
        row
    }

    fn render_row(&self, cells: &[String], col_widths: &[usize], is_header: bool) -> String {
        let mut row = String::new();
        row.push_str(&"│".bright_cyan().bold().to_string());

        for (i, cell) in cells.iter().enumerate() {
            let width = col_widths.get(i).copied().unwrap_or(cell.len() + 2);
            let padding = width.saturating_sub(cell.len());
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;

            row.push_str(&" ".repeat(left_pad));

            if is_header {
                row.push_str(&cell.bright_green().bold().to_string());
            } else {
                // Alternate row colors for visual appeal
                row.push_str(&cell.white().to_string());
            }

            row.push_str(&" ".repeat(right_pad));

            if i < cells.len() - 1 {
                row.push_str(&"│".bright_cyan().to_string());
            }
        }

        row.push_str(&"│".bright_cyan().bold().to_string());
        row
    }
}

/// Helper function to create a status box with rounded corners
pub fn create_status_box(title: &str, items: Vec<(&str, &str)>, palette: &ColorPalette) -> String {
    let mut output = String::new();

    // Top border
    output.push_str(&"╭".bright_cyan().bold().to_string());
    output.push_str(&"─".repeat(58).bright_cyan().bold().to_string());
    output.push_str(&"╮".bright_cyan().bold().to_string());
    output.push('\n');

    // Title
    let title_padding = (58 - title.len()) / 2;
    output.push_str(&"│".bright_cyan().bold().to_string());
    output.push_str(&" ".repeat(title_padding));
    output.push_str(&title.bright_yellow().bold().to_string());
    output.push_str(&" ".repeat(58 - title.len() - title_padding));
    output.push_str(&"│".bright_cyan().bold().to_string());
    output.push('\n');

    // Separator
    output.push_str(&"├".bright_cyan().to_string());
    output.push_str(&"─".repeat(58).bright_cyan().to_string());
    output.push_str(&"┤".bright_cyan().to_string());
    output.push('\n');

    // Items
    for (label, value) in items {
        let item_line = format!("  {} {}", label, value);
        let padding = 58 - item_line.len() + 2; // Account for ANSI codes
        output.push_str(&"│".bright_cyan().bold().to_string());
        output.push_str(&item_line);
        output.push_str(&" ".repeat(padding.max(1)));
        output.push_str(&"│".bright_cyan().bold().to_string());
        output.push('\n');
    }

    // Bottom border
    output.push_str(&"╰".bright_cyan().bold().to_string());
    output.push_str(&"─".repeat(58).bright_cyan().bold().to_string());
    output.push_str(&"╯".bright_cyan().bold().to_string());

    output
}

/// Helper function to create a simple section header with rounded corners
pub fn create_section_header(title: &str) -> String {
    let mut output = String::new();
    output.push_str(&"┌".bright_magenta().bold().to_string());
    output.push_str(&"─".repeat(title.len() + 4).bright_magenta().bold().to_string());
    output.push_str(&"┐".bright_magenta().bold().to_string());
    output.push('\n');

    output.push_str(&"│ ".bright_magenta().bold().to_string());
    output.push_str(&title.bright_magenta().bold().to_string());
    output.push_str(&" │".bright_magenta().bold().to_string());
    output.push('\n');

    output.push_str(&"└".bright_magenta().bold().to_string());
    output.push_str(&"─".repeat(title.len() + 4).bright_magenta().bold().to_string());
    output.push_str(&"┘".bright_magenta().bold().to_string());

    output
}
