use chrono::{Local, Timelike};
use colored::*;
use std::thread;
use std::time::Duration;
use terminal_lock_core::types::{GreetingStyle, Quote};

use super::ColorPalette;

/// Greeting display handler
pub struct GreetingDisplay;

impl GreetingDisplay {
    /// Generate greeting text based on time and style
    pub fn generate_greeting(name: &str, style: GreetingStyle) -> String {
        let hour = Local::now().hour();
        let (time_phrase, title) = Self::get_time_title(hour);

        match style {
            GreetingStyle::Dynamic => {
                if hour >= 21 || hour < 5 {
                    format!("How's the night coffee with coding, {} {}?", title, name)
                } else {
                    format!("Good {}, {} {}!", time_phrase, title, name)
                }
            }
            GreetingStyle::Formal => format!("Good {}, {}.", time_phrase, name),
            GreetingStyle::Casual => format!("Hey {}! Ready to code?", name),
        }
    }

    /// Get time phrase and title based on hour
    fn get_time_title(hour: u32) -> (&'static str, &'static str) {
        match hour {
            5..=11 => ("morning", "Achiever"),
            12..=16 => ("afternoon", "Developer"),
            17..=20 => ("evening", "Cloud Engineer"),
            _ => ("night", "System Engineer"),
        }
    }

    /// Display full greeting with quote
    pub fn display(name: &str, style: GreetingStyle, quote: Quote, palette: &ColorPalette) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        // Top border
        let border = "═".repeat(59);
        println!("{}", palette.colorize(&border).bold());

        // Greeting with typewriter effect
        let greeting = Self::generate_greeting(name, style);
        Self::typewriter(&greeting, palette, 0.03);

        // Bottom border
        println!("{}", palette.colorize(&border).bold());

        // Quote
        println!();
        Self::typewriter(&format!("\"{}\"", quote.text), palette, 0.02);

        if let Some(author) = quote.author {
            println!("{}", format!("  — {}", author).cyan());
        }

        // Final border
        println!("\n{}\n", palette.colorize(&border).bold());
    }

    /// Typewriter effect for text
    fn typewriter(text: &str, palette: &ColorPalette, delay_secs: f32) {
        for ch in text.chars() {
            print!("{}", palette.colorize(&ch.to_string()).bold());
            use std::io::Write;
            std::io::stdout().flush().ok();
            thread::sleep(Duration::from_secs_f32(delay_secs));
        }
        println!();
    }

    /// Show loading animation
    pub fn show_loading(message: &str) {
        print!("{}", message.cyan());
        use std::io::Write;
        std::io::stdout().flush().ok();

        let spinner = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        for _ in 0..10 {
            for frame in &spinner {
                print!("\r{} {}", message.cyan(), frame.cyan());
                std::io::stdout().flush().ok();
                thread::sleep(Duration::from_millis(80));
            }
        }
        print!("\r{}\r", " ".repeat(message.len() + 2)); // Clear line
        std::io::stdout().flush().ok();
    }
}