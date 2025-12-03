use chrono::{Local, Timelike};
use colored::*;
use terminal_lock_config::{load_preferences, DisplayNameManager};
use terminal_lock_core::{error::*, types::GreetingStyle, Paths};
use terminal_lock_quotes::QuoteFetcher;
use terminal_lock_ui::ColorPalette;

pub async fn show_greeting(paths: &Paths) -> Result<()> {
    let prefs = load_preferences(paths)?;
    let palette = ColorPalette::new(prefs.color_scheme);
    let name = DisplayNameManager::load(paths)?;

    // Clear screen and show header
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .color(match prefs.color_scheme {
                terminal_lock_core::types::ColorScheme::Vibrant => Color::BrightCyan,
                terminal_lock_core::types::ColorScheme::Pastel => Color::Cyan,
                terminal_lock_core::types::ColorScheme::Minimal => Color::Green,
            })
            .bold()
    );

    // Generate greeting
    let greeting = generate_greeting(&name, prefs.greeting_style);
    println!("{}", palette.colorize(&greeting).bold());

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .color(match prefs.color_scheme {
                terminal_lock_core::types::ColorScheme::Vibrant => Color::BrightCyan,
                terminal_lock_core::types::ColorScheme::Pastel => Color::Cyan,
                terminal_lock_core::types::ColorScheme::Minimal => Color::Green,
            })
            .bold()
    );

    // Fetch and display quote
    if prefs.animations {
        println!("\n{}", "Fetching inspiration...".cyan());
    }

    let fetcher = QuoteFetcher::new();
    let quote = fetcher.fetch(prefs.quote_source).await;

    if prefs.animations {
        // Clear "Fetching..." line
        print!("\x1B[1A\x1B[2K");
    }

    println!("\n{}", format!("\"{}\"", quote.text).bold());
    if let Some(author) = quote.author {
        println!("{}", format!("  — {}", author).cyan());
    }

    println!(
        "\n{}\n",
        "═══════════════════════════════════════════════════════════"
            .color(match prefs.color_scheme {
                terminal_lock_core::types::ColorScheme::Vibrant => Color::BrightCyan,
                terminal_lock_core::types::ColorScheme::Pastel => Color::Cyan,
                terminal_lock_core::types::ColorScheme::Minimal => Color::Green,
            })
            .bold()
    );

    Ok(())
}

fn generate_greeting(name: &str, style: GreetingStyle) -> String {
    let hour = Local::now().hour();
    let (time_phrase, title) = match hour {
        5..=11 => ("morning", "Achiever"),
        12..=16 => ("afternoon", "Developer"),
        17..=20 => ("evening", "Cloud Engineer"),
        _ => ("night", "System Engineer"),
    };

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