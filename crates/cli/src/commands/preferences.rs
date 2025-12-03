use colored::*;
use std::path::PathBuf;
use terminal_lock_config::{ConfigManager, DisplayNameManager};
use terminal_lock_core::{
    error::*,
    types::{ColorScheme, GreetingStyle, QuoteSource, AnimationType},
    Paths,
};
use terminal_lock_ui::PasswordPrompt;

pub async fn run(paths: &Paths) -> Result<()> {
    let mut manager = ConfigManager::new(paths.clone())?;
    let term = console::Term::stderr();

    loop {
        term.clear_screen().ok();

        // Show current settings
        println!("{}", "╔═══════════════════════════════════════════════════════╗".blue().bold());
        println!("{}", "║      Terminal Lock - Preference Manager              ║".blue().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════╝".blue().bold());
        println!();

        println!("{}\n", "Current Settings:".cyan().bold());

        let name = DisplayNameManager::load(paths)?;
        let prefs = manager.preferences();

        println!("{}      {}", "Display Name:".green(), name);
        println!("{}      {}", "Color Scheme:".green(), prefs.color_scheme);
        println!("{}        {}", "Animations:".green(), prefs.animations);
        if prefs.animations {
            let anim_name = match prefs.animation_type {
                AnimationType::Matrix => "Matrix",
                AnimationType::Starfield => "Starfield",
                AnimationType::Particles => "Particles",
            };
            println!("{}   {}", "Animation Type:".green(), anim_name);
        }
        println!("{}      {}", "Quote Source:".green(), prefs.quote_source);
        println!("{}    {}", "Greeting Style:".green(), prefs.greeting_style);
        println!();

        // Menu
        let options = vec![
            "Change Display Name",
            "Change Color Scheme",
            "Toggle Animations",
            "Change Animation Type",
            "Change Quote Source",
            "Change Greeting Style",
            "Reset to Defaults",
            "Export Preferences",
            "Import Preferences",
            "Exit",
        ];

        let choice = PasswordPrompt::show_menu("Choose an option:", &options)?;

        match choice {
            0 => change_display_name(paths)?,
            1 => change_color_scheme(&mut manager)?,
            2 => toggle_animations(&mut manager)?,
            3 => change_animation_type(&mut manager)?,
            4 => change_quote_source(&mut manager)?,
            5 => change_greeting_style(&mut manager)?,
            6 => reset_to_defaults(&mut manager)?,
            7 => export_preferences(&manager)?,
            8 => import_preferences(&mut manager)?,
            9 => {
                println!("\n{}", "Preferences saved. Changes will take effect on next terminal session.".green());
                return Ok(());
            }
            _ => {}
        }

        println!("\n{}", "Press Enter to continue...".cyan());
        PasswordPrompt::read_line("")?;
    }
}

fn change_display_name(paths: &Paths) -> Result<()> {
    let current = DisplayNameManager::load(paths)?;
    println!("\n{}", format!("Current: {}", current).green());

    let new_name = PasswordPrompt::read_line("Enter new display name: ")?;
    
    if !new_name.is_empty() {
        DisplayNameManager::save(paths, &new_name)?;
        println!("{}", format!("✓ Display name updated to: {}", new_name).green());
    }

    Ok(())
}

fn change_color_scheme(manager: &mut ConfigManager) -> Result<()> {
    let schemes = vec!["Vibrant", "Pastel", "Minimal"];
    let choice = PasswordPrompt::show_menu("Choose color scheme:", &schemes)?;

    let scheme = match choice {
        0 => ColorScheme::Vibrant,
        1 => ColorScheme::Pastel,
        2 => ColorScheme::Minimal,
        _ => ColorScheme::Vibrant,
    };

    manager.set_color_scheme(scheme)?;
    println!("{}", format!("✓ Color scheme updated to: {}", scheme).green());

    Ok(())
}

fn toggle_animations(manager: &mut ConfigManager) -> Result<()> {
    let current = manager.preferences().animations;
    let new_value = !current;

    manager.set_animations(new_value)?;
    println!(
        "{}",
        format!("✓ Animations set to: {}", new_value).green()
    );

    Ok(())
}

fn change_animation_type(manager: &mut ConfigManager) -> Result<()> {
    let types = vec!["Matrix", "Starfield", "Particles"];
    let choice = PasswordPrompt::show_menu("Choose animation type:", &types)?;

    let anim_type = match choice {
        0 => AnimationType::Matrix,
        1 => AnimationType::Starfield,
        2 => AnimationType::Particles,
        _ => AnimationType::Matrix,
    };

    manager.set_animation_type(anim_type)?;
    let anim_name = match anim_type {
        AnimationType::Matrix => "Matrix",
        AnimationType::Starfield => "Starfield",
        AnimationType::Particles => "Particles",
    };
    println!("{}", format!("✓ Animation type updated to: {}", anim_name).green());

    Ok(())
}

fn change_quote_source(manager: &mut ConfigManager) -> Result<()> {
    let sources = vec!["All", "Coding/Tech", "Spiritual", "Motivational", "General"];
    let choice = PasswordPrompt::show_menu("Choose quote source:", &sources)?;

    let source = match choice {
        0 => QuoteSource::All,
        1 => QuoteSource::Coding,
        2 => QuoteSource::Spiritual,
        3 => QuoteSource::Motivational,
        4 => QuoteSource::General,
        _ => QuoteSource::All,
    };

    manager.set_quote_source(source)?;
    println!("{}", format!("✓ Quote source updated to: {}", source).green());

    Ok(())
}

fn change_greeting_style(manager: &mut ConfigManager) -> Result<()> {
    let styles = vec!["Dynamic", "Formal", "Casual"];
    let choice = PasswordPrompt::show_menu("Choose greeting style:", &styles)?;

    let style = match choice {
        0 => GreetingStyle::Dynamic,
        1 => GreetingStyle::Formal,
        2 => GreetingStyle::Casual,
        _ => GreetingStyle::Dynamic,
    };

    manager.set_greeting_style(style)?;
    println!("{}", format!("✓ Greeting style updated to: {}", style).green());

    Ok(())
}

fn reset_to_defaults(manager: &mut ConfigManager) -> Result<()> {
    let confirm = PasswordPrompt::confirm(
        "Are you sure you want to reset all preferences to defaults?",
        false,
    )?;

    if confirm {
        manager.reset_to_defaults()?;
        println!("{}", "✓ Preferences reset to defaults".green());
    } else {
        println!("{}", "Reset cancelled".yellow());
    }

    Ok(())
}

fn export_preferences(manager: &ConfigManager) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| Error::Unknown("Cannot find home directory".into()))?;
    let export_path = home.join("terminal-lock-prefs-backup.json");

    manager.export_to_file(&export_path)?;
    println!("{}", "✓ Preferences exported to:".green());
    println!("  {}", export_path.display());

    Ok(())
}

fn import_preferences(manager: &mut ConfigManager) -> Result<()> {
    let path_str = PasswordPrompt::read_line("Enter path to preferences file: ")?;
    let path = PathBuf::from(path_str);

    if !path.exists() {
        return Err(Error::Unknown(format!("File not found: {}", path.display())));
    }

    manager.import_from_file(&path)?;
    println!("{}", "✓ Preferences imported successfully".green());

    Ok(())
}