/// Preference management with database backend
use terminal_lock_core::error::*;
use terminal_lock_core::types::{ColorScheme, GreetingStyle, QuoteSource};
use terminal_lock_ipc::{DaemonClient, DaemonRequest, DaemonResponse};

/// Save user preferences to daemon database
pub fn save_preferences_to_db(
    user_id: u32,
    display_name: &str,
    color_scheme: &str,
    animations: bool,
    animation_type: &str,
    quote_source: &str,
    greeting_style: &str,
) -> Result<()> {
    let request = DaemonRequest::SavePreferences {
        user_id,
        display_name: display_name.to_string(),
        color_scheme: color_scheme.to_string(),
        animations,
        animation_type: animation_type.to_string(),
        quote_source: quote_source.to_string(),
        greeting_style: greeting_style.to_string(),
    };
    
    match DaemonClient::send_message(request) {
        Ok(DaemonResponse::PreferencesSaved { user_id: _ }) => {
            Ok(())
        }
        Ok(_) => {
            Err(Error::Unknown("Unexpected daemon response".to_string()))
        }
        Err(e) => {
            eprintln!("⚠️  Cannot save preferences: {}", e);
            Err(Error::Unknown(format!("IPC error: {}", e)))
        }
    }
}

/// Get user preferences from daemon database (returns strings)
pub fn get_preferences_from_db(user_id: u32) -> Result<Option<(String, String, bool, String, String, String)>> {
    let request = DaemonRequest::GetPreferences { user_id };
    
    match DaemonClient::send_message(request) {
        Ok(DaemonResponse::Preferences {
            display_name,
            color_scheme,
            animations,
            animation_type,
            quote_source,
            greeting_style,
            ..
        }) => {
            Ok(Some((display_name, color_scheme, animations, animation_type, quote_source, greeting_style)))
        }
        Ok(_) => {
            Err(Error::Unknown("Unexpected daemon response".to_string()))
        }
        Err(e) => {
            eprintln!("⚠️  Cannot get preferences: {}", e);
            Err(Error::Unknown(format!("IPC error: {}", e)))
        }
    }
}

/// Get color scheme from string
pub fn parse_color_scheme(scheme_str: &str) -> ColorScheme {
    match scheme_str {
        "pastel" => ColorScheme::Pastel,
        "minimal" => ColorScheme::Minimal,
        _ => ColorScheme::Vibrant,
    }
}

/// Get greeting style from string
pub fn parse_greeting_style(style_str: &str) -> GreetingStyle {
    match style_str {
        "formal" => GreetingStyle::Formal,
        "casual" => GreetingStyle::Casual,
        _ => GreetingStyle::Dynamic,
    }
}

/// Get quote source from string
pub fn parse_quote_source(source_str: &str) -> QuoteSource {
    match source_str {
        "coding" => QuoteSource::Coding,
        // "motivation" => QuoteSource::Motivation,
        // "philosophy" => QuoteSource::Philosophy,
        "spiritual" => QuoteSource::Spiritual,
        // "all" => QuoteSource::All,
        _ => QuoteSource::All,
    }
}
