use colored::*;
use terminal_lock_auth::{
    lockout::AttemptsTracker,
    password::{PasswordHasher, PasswordValidator},
};
use terminal_lock_config::{save_preferences, DisplayNameManager};
use terminal_lock_core::{
    error::*,
    types::{ColorScheme, GreetingStyle, Preferences, QuoteSource, AnimationType},
    Paths,
};
use terminal_lock_ui::PasswordPrompt;

pub async fn run(paths: &Paths) -> Result<()> {
    let term = console::Term::stderr();
    term.clear_screen().ok();

    // Header
    println!("{}", "╔═══════════════════════════════════════════════════════╗".blue().bold());
    println!("{}", "║   Terminal Lock Setup - Enhanced with Animations     ║".blue().bold());
    println!("{}", "║     1000+ Colors | 43 Working APIs | Background FX   ║".blue().bold());
    println!("{}", "║     15min Sessions | Per-Terminal + Global Tracking  ║".blue().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════╝".blue().bold());
    println!();

    // Step 1: Display Name
    println!("{}", "Step 1: Display Name".cyan().bold());
    let display_name = loop {
        let name = PasswordPrompt::read_line("How would you like to be greeted? (e.g., Moses, DevMoses): ")?;
        
        if name.is_empty() {
            eprintln!("{}", "Name cannot be empty".red());
            continue;
        }
        
        break name;
    };
    
    DisplayNameManager::save(paths, &display_name)?;
    println!("{}\n", format!("✓ Display name set to: {}", display_name).green());

    // Step 2: Password
    println!("{}", "Step 2: Password Setup".cyan().bold());
    println!("Password must be 6 characters with:");
    println!("  • At least 1 uppercase letter (A-Z)");
    println!("  • At least 1 lowercase letter (a-z)");
    println!("  • At least 1 digit (0-9)");
    println!();

    let password = loop {
        let pw = PasswordPrompt::prompt("Enter password: ")?;
        let confirm = PasswordPrompt::prompt("Confirm password: ")?;

        if pw != confirm {
            eprintln!("{}\n", "✗ Passwords don't match. Try again.".red());
            continue;
        }

        match PasswordValidator::validate(&pw) {
            Ok(_) => break pw,
            Err(e) => {
                eprintln!("{}\n", format!("✗ {}", e).red());
                continue;
            }
        }
    };

    PasswordHasher::save_password(paths, &password)?;
    println!("{}\n", "✓ Password set securely (Argon2 + salt)".green());

    // Step 3: Color Scheme
    println!("{}", "Step 3: Visual Preferences".cyan().bold());
    println!("Color scheme:");
    println!("  1) Vibrant (1000+ colors, very dynamic)");
    println!("  2) Pastel (soft, easy on eyes)");
    println!("  3) Minimal (4 colors, simple)");

    let color_scheme = match PasswordPrompt::show_menu("Choose color scheme", &["Vibrant", "Pastel", "Minimal"])? {
        0 => ColorScheme::Vibrant,
        1 => ColorScheme::Pastel,
        2 => ColorScheme::Minimal,
        _ => ColorScheme::Vibrant,
    };

    // Step 4: Animations
    let animations = PasswordPrompt::confirm("Enable background animations?", true)?;
    
    let animation_type = if animations {
        println!("\nAnimation style:");
        println!("  1) Matrix (green falling characters)");
        println!("  2) Starfield (stars moving across screen)");
        println!("  3) Particles (colored bouncing dots)");
        
        match PasswordPrompt::show_menu(
            "Choose animation style",
            &["Matrix", "Starfield", "Particles"],
        )? {
            0 => AnimationType::Matrix,
            1 => AnimationType::Starfield,
            2 => AnimationType::Particles,
            _ => AnimationType::Matrix,
        }
    } else {
        AnimationType::Matrix // Default even if disabled
    };

    // Step 5: Quote Source
    println!("\nQuote source (43 working APIs):");
    println!("  1) All - Maximum variety (43 APIs)");
    println!("  2) Coding/Tech (4 APIs)");
    println!("  3) Spiritual/Wisdom (8 APIs)");
    println!("  4) Motivational (3 APIs)");
    println!("  5) General/Famous (12 APIs)");

    let quote_source = match PasswordPrompt::show_menu(
        "Choose quote source",
        &["All", "Coding/Tech", "Spiritual", "Motivational", "General"],
    )? {
        0 => QuoteSource::All,
        1 => QuoteSource::Coding,
        2 => QuoteSource::Spiritual,
        3 => QuoteSource::Motivational,
        4 => QuoteSource::General,
        _ => QuoteSource::All,
    };

    // Step 6: Greeting Style
    println!("\nGreeting style:");
    println!("  1) Dynamic (changes with time)");
    println!("  2) Formal (professional)");
    println!("  3) Casual (friendly)");

    let greeting_style = match PasswordPrompt::show_menu(
        "Choose greeting style",
        &["Dynamic", "Formal", "Casual"],
    )? {
        0 => GreetingStyle::Dynamic,
        1 => GreetingStyle::Formal,
        2 => GreetingStyle::Casual,
        _ => GreetingStyle::Dynamic,
    };

    // Save preferences
    let preferences = Preferences {
        color_scheme,
        animations,
        animation_type,
        quote_source,
        greeting_style,
    };
    save_preferences(paths, &preferences)?;

    // Initialize attempt counter
    AttemptsTracker::reset(paths)?;

    // Summary
    term.clear_screen().ok();
    println!("\n{}", "✓ Setup complete!".green().bold());
    println!("\n{}", "Security Features:".cyan().bold());
    println!("  • Each terminal session lasts 15 minutes");
    println!("  • Each terminal has 3 attempts (60s lockout)");
    println!("  • Global counter tracks 9 total attempts");
    println!("  • After 9 failed attempts, laptop locks");
    println!("  • Counter resets when laptop is unlocked");
    println!("  • Background animations (stars, matrix, particles)");
    println!("\n{}", "Next terminal will require authentication.".cyan());
    println!();

    Ok(())
}