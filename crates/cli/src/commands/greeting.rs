use chrono::{Local, Timelike};
use colored::*;
use terminal_lock_config;
use terminal_lock_core::{error::*, types::{GreetingStyle, ColorScheme}, Paths};
use terminal_lock_quotes::QuoteFetcher;
use terminal_lock_ui::ColorPalette;
use crate::ipc_helper;

pub async fn show_greeting(paths: &Paths) -> Result<()> {
    // Load preferences from database via IPC (delegated to config crate)
    let (name, color_scheme_str, _, _, _, greeting_style_str) = match ipc_helper::get_preferences(1) {
        Ok(Some(prefs)) => prefs,
        _ => ("User".to_string(), "vibrant".to_string(), true, "matrix".to_string(), "all".to_string(), "dynamic".to_string()),
    };
    
    let color_scheme = terminal_lock_config::parse_color_scheme(&color_scheme_str);
    let palette = ColorPalette::new(color_scheme);
    let greeting_style = terminal_lock_config::parse_greeting_style(&greeting_style_str);

    // Clear screen and show header
    print!("\x1B[2J\x1B[1;1H"); // Clear screen
    
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .color(match color_scheme {
                ColorScheme::Vibrant => Color::BrightCyan,
                ColorScheme::Pastel => Color::Cyan,
                ColorScheme::Minimal => Color::Green,
            })
            .bold()
    );

    // Generate greeting
    let greeting = generate_greeting(&name, greeting_style);
    println!("{}", palette.colorize(&greeting).bold());

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════"
            .color(match color_scheme {
                ColorScheme::Vibrant => Color::BrightCyan,
                ColorScheme::Pastel => Color::Cyan,
                ColorScheme::Minimal => Color::Green,
            })
            .bold()
    );

    // Fetch and display quote
    let animations = match ipc_helper::get_preferences(1) {
        Ok(Some((_, _, anim, _, _, _))) => anim,
        _ => true,
    };
    
    if animations {
        println!("\n{}", "Fetching inspiration...".cyan());
    }

    let fetcher = QuoteFetcher::new();
    let quote_source_str = match ipc_helper::get_preferences(1) {
        Ok(Some((_, _, _, _, qs, _))) => qs,
        _ => "all".to_string(),
    };
    let quote_source = terminal_lock_config::parse_quote_source(&quote_source_str);
    
    let quote = fetcher.fetch(quote_source).await;

    if animations {
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
            .color(match color_scheme {
                ColorScheme::Vibrant => Color::BrightCyan,
                ColorScheme::Pastel => Color::Cyan,
                ColorScheme::Minimal => Color::Green,
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