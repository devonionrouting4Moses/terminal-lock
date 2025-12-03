/// Example: Custom authentication flow
use terminal_lock_auth::Authenticator;
use terminal_lock_core::Paths;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize paths
    let paths = Paths::new()?;
    paths.ensure_lock_dir()?;

    // Create authenticator
    let authenticator = Authenticator::new(paths);

    // Check if authentication is needed
    if authenticator.needs_authentication()? {
        println!("Authentication required!");

        // Custom password prompt
        let result = authenticator.authenticate(|| {
            print!("Enter your password: ");
            use std::io::Write;
            std::io::stdout().flush()?;
            
            let mut password = String::new();
            std::io::stdin().read_line(&mut password)?;
            Ok(password.trim().to_string())
        });

        match result {
            Ok(_) => println!("✓ Authentication successful!"),
            Err(e) => eprintln!("✗ Authentication failed: {}", e),
        }
    } else {
        println!("Already authenticated - session is valid!");
    }

    Ok(())
}