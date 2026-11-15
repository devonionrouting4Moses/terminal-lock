use colored::*;
use std::path::PathBuf;
use terminal_lock_core::{
    error::*,
    Paths,
    types::ColorScheme,
};
use terminal_lock_ui::{PasswordPrompt, Table};
use terminal_lock_core::types::ColorScheme as CoreColorScheme;
use crate::ipc_helper;

pub async fn run(paths: &Paths) -> Result<()> {
    let term = console::Term::stderr();

    loop {
        // Info box
        let info_table = Table::new(terminal_lock_ui::ColorPalette::new(ColorScheme::Vibrant))
            .with_title("   Terminal Lock - Preference Manager   ")
            .with_headers(vec!["Info"])
            .add_row(vec!["ℹ For more information, run: terminal-lock <command> --help"])
            .add_row(vec!["🔒 Session-based security for your terminal"])
            .render();
        
        println!("{}", info_table);
        println!();

        // Load from daemon database
        let (name, color_scheme_str, animations, animation_type_str, quote_source_str, greeting_style_str) = 
            match ipc_helper::get_preferences(1) {
                Ok(Some((n, cs, a, at, qs, gs))) => (n, cs, a, at, qs, gs),
                _ => {
                    println!("{}", "Error: Could not load preferences from database".red());
                    println!("{}", "Please ensure the daemon is running and preferences are initialized.".yellow());
                    return Err(Error::Unknown("Failed to load preferences from database".to_string()));
                }
            };

        let anim_name = if animations {
            match animation_type_str.as_str() {
                "matrix" => "Matrix",
                "starfield" => "Starfield",
                "particles" => "Particles",
                _ => "Unknown",
            }
        } else {
            "Disabled"
        };

        // Current Settings Table
        let table = Table::new(terminal_lock_ui::ColorPalette::new(CoreColorScheme::Vibrant))
            .with_title("Current Settings")
            .with_headers(vec!["Setting", "Value"])
            .add_row(vec!["Display Name", &name])
            .add_row(vec!["Color Scheme", &color_scheme_str])
            .add_row(vec!["Animations", if animations { "Enabled" } else { "Disabled" }])
            .add_row(vec!["Animation Type", anim_name])
            .add_row(vec!["Quote Source", &quote_source_str])
            .add_row(vec!["Greeting Style", &greeting_style_str])
            .render();
        
        println!("{}", table);
        println!();

        // Menu Options Table
        let menu_table = Table::new(terminal_lock_ui::ColorPalette::new(CoreColorScheme::Vibrant))
            .with_title("Available Options")
            .with_headers(vec!["#", "Action"])
            .add_row(vec!["1", "Change Display Name"])
            .add_row(vec!["2", "Change Color Scheme"])
            .add_row(vec!["3", "Toggle Animations"])
            .add_row(vec!["4", "Change Animation Type"])
            .add_row(vec!["5", "Change Quote Source"])
            .add_row(vec!["6", "Change Greeting Style"])
            .add_row(vec!["7", "Reset to Defaults"])
            .add_row(vec!["8", "Export Preferences"])
            .add_row(vec!["9", "Import Preferences"])
            .add_row(vec!["10", "Exit"])
            .render();
        
        println!("{}", menu_table);
        println!();

        let choice_str = PasswordPrompt::read_line("Choose (1-10): ")?;
        let choice: usize = match choice_str.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= 10 => n - 1,
            _ => {
                println!("{}", "Invalid choice. Please try again.".red());
                PasswordPrompt::read_line("Press Enter to continue...")?;
                continue;
            }
        };

        match choice {
            0 => change_display_name()?,
            1 => change_color_scheme()?,
            2 => toggle_animations(&color_scheme_str, animations, &animation_type_str, &quote_source_str, &greeting_style_str)?,
            3 => change_animation_type(&name, &color_scheme_str, animations, &quote_source_str, &greeting_style_str)?,
            4 => change_quote_source(&name, &color_scheme_str, animations, &animation_type_str, &greeting_style_str)?,
            5 => change_greeting_style(&name, &color_scheme_str, animations, &animation_type_str, &quote_source_str)?,
            6 => reset_to_defaults()?,
            7 => export_preferences()?,
            8 => import_preferences()?,
            9 => {
                println!("\n{}", "✓ Preferences saved. Changes will take effect on next terminal session.".green());
                return Ok(());
            }
            _ => {}
        }

        println!("\n{}", "Press Enter to continue...".cyan());
        PasswordPrompt::read_line("")?;
    }
}

fn change_display_name() -> Result<()> {
    let table = Table::new(terminal_lock_ui::ColorPalette::new(CoreColorScheme::Vibrant))
        .with_title("Change Display Name")
        .with_headers(vec!["Action"])
        .add_row(vec!["Enter your new display name"])
        .render();
    
    println!("{}", table);
    println!();

    let new_name = PasswordPrompt::read_line("Enter new display name: ")?;
    
    if !new_name.is_empty() {
        // Get current preferences from database
        if let Ok(Some((_, color_scheme_str, animations, animation_type_str, quote_source_str, greeting_style_str))) = ipc_helper::get_preferences(1) {
            // Save updated preferences to daemon
            ipc_helper::save_preferences(
                1,
                &new_name,
                &color_scheme_str,
                animations,
                &animation_type_str,
                &quote_source_str,
                &greeting_style_str,
            )?;
            println!("{}", format!("✓ Display name updated to: {}", new_name).green());
        } else {
            println!("{}", "Error: Could not load current preferences from database".red());
        }
    }

    Ok(())
}

fn change_color_scheme() -> Result<()> {
    let table = Table::new(terminal_lock_ui::ColorPalette::new(CoreColorScheme::Vibrant))
        .with_title("Choose Color Scheme")
        .with_headers(vec!["#", "Scheme"])
        .add_row(vec!["1", "Vibrant"])
        .add_row(vec!["2", "Pastel"])
        .add_row(vec!["3", "Minimal"])
        .render();
    
    println!("{}", table);
    println!();
    
    let choice_str = PasswordPrompt::read_line("Choose (1-3): ")?;
    let choice: usize = match choice_str.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= 3 => n - 1,
        _ => {
            println!("{}", "Invalid choice.".red());
            return Ok(());
        }
    };

    let scheme_str = match choice {
        0 => "vibrant",
        1 => "pastel",
        2 => "minimal",
        _ => "vibrant",
    };

    // Get current preferences from database
    if let Ok(Some((name, _, animations, animation_type_str, quote_source_str, greeting_style_str))) = ipc_helper::get_preferences(1) {
        // Save updated preferences to daemon
        ipc_helper::save_preferences(
            1,
            &name,
            scheme_str,
            animations,
            &animation_type_str,
            &quote_source_str,
            &greeting_style_str,
        )?;
        println!("{}", format!("✓ Color scheme updated to: {}", scheme_str).green());
    } else {
        println!("{}", "Error: Could not load current preferences from database".red());
    }

    Ok(())
}

fn toggle_animations(color_scheme_str: &str, animations: bool, animation_type_str: &str, quote_source_str: &str, greeting_style_str: &str) -> Result<()> {
    let new_value = !animations;

    // Get current preferences from database
    if let Ok(Some((name, _, _, _, _, _))) = ipc_helper::get_preferences(1) {
        // Save updated preferences to daemon
        ipc_helper::save_preferences(
            1,
            &name,
            color_scheme_str,
            new_value,
            animation_type_str,
            quote_source_str,
            greeting_style_str,
        )?;
        println!(
            "{}",
            format!("✓ Animations set to: {}", new_value).green()
        );
    } else {
        println!("{}", "Error: Could not load current preferences from database".red());
    }

    Ok(())
}

fn change_animation_type(name: &str, color_scheme_str: &str, animations: bool, quote_source_str: &str, greeting_style_str: &str) -> Result<()> {
    let table = Table::new(terminal_lock_ui::ColorPalette::new(CoreColorScheme::Vibrant))
        .with_title("Choose Animation Type")
        .with_headers(vec!["#", "Type"])
        .add_row(vec!["1", "Matrix"])
        .add_row(vec!["2", "Starfield"])
        .add_row(vec!["3", "Particles"])
        .render();
    
    println!("{}", table);
    println!();
    
    let choice_str = PasswordPrompt::read_line("Choose (1-3): ")?;
    let choice: usize = match choice_str.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= 3 => n - 1,
        _ => {
            println!("{}", "Invalid choice.".red());
            return Ok(());
        }
    };

    let anim_type_str = match choice {
        0 => "matrix",
        1 => "starfield",
        2 => "particles",
        _ => "matrix",
    };

    let anim_name = match choice {
        0 => "Matrix",
        1 => "Starfield",
        2 => "Particles",
        _ => "Matrix",
    };
    
    // Save updated preferences to daemon
    ipc_helper::save_preferences(
        1,
        name,
        color_scheme_str,
        animations,
        anim_type_str,
        quote_source_str,
        greeting_style_str,
    )?;
    println!("{}", format!("✓ Animation type updated to: {}", anim_name).green());

    Ok(())
}

fn change_quote_source(name: &str, color_scheme_str: &str, animations: bool, animation_type_str: &str, greeting_style_str: &str) -> Result<()> {
    let table = Table::new(terminal_lock_ui::ColorPalette::new(CoreColorScheme::Vibrant))
        .with_title("Choose Quote Source")
        .with_headers(vec!["#", "Source"])
        .add_row(vec!["1", "All"])
        .add_row(vec!["2", "Coding/Tech"])
        .add_row(vec!["3", "Spiritual"])
        .add_row(vec!["4", "Motivational"])
        .add_row(vec!["5", "General"])
        .render();
    
    println!("{}", table);
    println!();
    
    let choice_str = PasswordPrompt::read_line("Choose (1-5): ")?;
    let choice: usize = match choice_str.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= 5 => n - 1,
        _ => {
            println!("{}", "Invalid choice.".red());
            return Ok(());
        }
    };

    let source_str = match choice {
        0 => "all",
        1 => "coding",
        2 => "spiritual",
        3 => "motivational",
        4 => "general",
        _ => "all",
    };

    // Save updated preferences to daemon
    ipc_helper::save_preferences(
        1,
        name,
        color_scheme_str,
        animations,
        animation_type_str,
        source_str,
        greeting_style_str,
    )?;
    println!("{}", format!("✓ Quote source updated to: {}", source_str).green());

    Ok(())
}

fn change_greeting_style(name: &str, color_scheme_str: &str, animations: bool, animation_type_str: &str, quote_source_str: &str) -> Result<()> {
    let table = Table::new(terminal_lock_ui::ColorPalette::new(CoreColorScheme::Vibrant))
        .with_title("Choose Greeting Style")
        .with_headers(vec!["#", "Style"])
        .add_row(vec!["1", "Dynamic"])
        .add_row(vec!["2", "Formal"])
        .add_row(vec!["3", "Casual"])
        .render();
    
    println!("{}", table);
    println!();
    
    let choice_str = PasswordPrompt::read_line("Choose (1-3): ")?;
    let choice: usize = match choice_str.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= 3 => n - 1,
        _ => {
            println!("{}", "Invalid choice.".red());
            return Ok(());
        }
    };

    let style_str = match choice {
        0 => "dynamic",
        1 => "formal",
        2 => "casual",
        _ => "dynamic",
    };

    // Save updated preferences to daemon
    ipc_helper::save_preferences(
        1,
        name,
        color_scheme_str,
        animations,
        animation_type_str,
        quote_source_str,
        style_str,
    )?;
    println!("{}", format!("✓ Greeting style updated to: {}", style_str).green());

    Ok(())
}

fn reset_to_defaults() -> Result<()> {
    let confirm = PasswordPrompt::confirm(
        "Are you sure you want to reset all preferences to defaults?",
        false,
    )?;

    if confirm {
        // Reset to default values in database
        ipc_helper::save_preferences(
            1,
            "User",
            "vibrant",
            true,
            "matrix",
            "all",
            "dynamic",
        )?;
        println!("{}", "✓ Preferences reset to defaults".green());
    } else {
        println!("{}", "Reset cancelled".yellow());
    }

    Ok(())
}

fn export_preferences() -> Result<()> {
    if let Ok(Some((name, color_scheme_str, animations, animation_type_str, quote_source_str, greeting_style_str))) = ipc_helper::get_preferences(1) {
        let home = dirs::home_dir().ok_or_else(|| Error::Unknown("Cannot find home directory".into()))?;
        let export_path = home.join("terminal-lock-prefs-backup.json");
        
        let prefs_json = serde_json::json!({
            "display_name": name,
            "color_scheme": color_scheme_str,
            "animations": animations,
            "animation_type": animation_type_str,
            "quote_source": quote_source_str,
            "greeting_style": greeting_style_str,
        });
        
        std::fs::write(&export_path, serde_json::to_string_pretty(&prefs_json)?)?;
        println!("{}", "✓ Preferences exported to:".green());
        println!("  {}", export_path.display());
    } else {
        println!("{}", "Error: Could not load preferences from database".red());
    }

    Ok(())
}

fn import_preferences() -> Result<()> {
    let path_str = PasswordPrompt::read_line("Enter path to preferences file: ")?;
    let path = PathBuf::from(path_str);

    if !path.exists() {
        return Err(Error::Unknown(format!("File not found: {}", path.display())));
    }

    let content = std::fs::read_to_string(&path)?;
    let prefs_json: serde_json::Value = serde_json::from_str(&content)?;
    
    let name = prefs_json["display_name"].as_str().unwrap_or("User");
    let color_scheme = prefs_json["color_scheme"].as_str().unwrap_or("vibrant");
    let animations = prefs_json["animations"].as_bool().unwrap_or(true);
    let animation_type = prefs_json["animation_type"].as_str().unwrap_or("matrix");
    let quote_source = prefs_json["quote_source"].as_str().unwrap_or("all");
    let greeting_style = prefs_json["greeting_style"].as_str().unwrap_or("dynamic");
    
    ipc_helper::save_preferences(
        1,
        name,
        color_scheme,
        animations,
        animation_type,
        quote_source,
        greeting_style,
    )?;
    println!("{}", "✓ Preferences imported successfully".green());

    Ok(())
}