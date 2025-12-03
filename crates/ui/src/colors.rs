use colored::*;
use rand::Rng;
use terminal_lock_core::types::ColorScheme;

/// Color palette generator
pub struct ColorPalette {
    scheme: ColorScheme,
}

impl ColorPalette {
    pub fn new(scheme: ColorScheme) -> Self {
        Self { scheme }
    }

    /// Get a random color string based on scheme
    pub fn random_color(&self) -> ColoredString {
        let mut rng = rand::rng();

        match self.scheme {
            ColorScheme::Vibrant => self.vibrant_color(&mut rng),
            ColorScheme::Pastel => self.pastel_color(&mut rng),
            ColorScheme::Minimal => self.minimal_color(&mut rng),
        }
    }

    fn vibrant_color(&self, rng: &mut impl Rng) -> ColoredString {
        let colors = vec![
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::BrightRed,
            Color::BrightGreen,
            Color::BrightYellow,
            Color::BrightBlue,
            Color::BrightMagenta,
            Color::BrightCyan,
        ];

        let color = colors[rng.random_range(0..colors.len())];
        "█".color(color)
    }

    fn pastel_color(&self, rng: &mut impl Rng) -> ColoredString {
        // Pastel using truecolor RGB
        let pastels = vec![
            (255, 182, 193), // Light pink
            (255, 218, 185), // Peach
            (255, 253, 208), // Cream
            (224, 255, 255), // Light cyan
            (230, 230, 250), // Lavender
            (255, 240, 245), // Lavender blush
        ];

        let (r, g, b) = pastels[rng.random_range(0..pastels.len())];
        "█".truecolor(r, g, b)
    }

    fn minimal_color(&self, rng: &mut impl Rng) -> ColoredString {
        let colors = [Color::Green, Color::Blue, Color::Cyan, Color::Magenta];
        let color = colors[rng.random_range(0..colors.len())];
        "█".color(color)
    }

    /// Get a colored border character
    pub fn border(&self) -> ColoredString {
        self.random_color()
    }

    /// Color a string
    pub fn colorize(&self, text: &str) -> ColoredString {
        let color = match self.scheme {
            ColorScheme::Vibrant => Color::BrightCyan,
            ColorScheme::Pastel => Color::Cyan,
            ColorScheme::Minimal => Color::Green,
        };
        text.color(color)
    }
}

/// Helper function to create a colored header
pub fn create_header(text: &str, palette: &ColorPalette) -> String {
    let border = "═".repeat(60);
    format!(
        "{}\n{}\n{}",
        palette.border().to_string() + &border,
        palette.colorize(text).bold(),
        palette.border().to_string() + &border
    )
}