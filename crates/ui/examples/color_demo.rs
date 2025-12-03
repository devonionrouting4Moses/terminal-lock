/// Example: Demonstrating color palette variations
use terminal_lock_core::types::ColorScheme;
use terminal_lock_ui::ColorPalette;

fn main() {
    println!("Terminal Lock - Color Palette Demo\n");

    // Vibrant colors
    println!("=== Vibrant Scheme (1000+ colors) ===");
    let vibrant = ColorPalette::new(ColorScheme::Vibrant);
    for _ in 0..20 {
        print!("{}", vibrant.random_color());
    }
    println!("\n");

    // Pastel colors
    println!("=== Pastel Scheme ===");
    let pastel = ColorPalette::new(ColorScheme::Pastel);
    for _ in 0..20 {
        print!("{}", pastel.random_color());
    }
    println!("\n");

    // Minimal colors
    println!("=== Minimal Scheme ===");
    let minimal = ColorPalette::new(ColorScheme::Minimal);
    for _ in 0..20 {
        print!("{}", minimal.random_color());
    }
    println!("\n");

    // Border examples
    println!("=== Borders ===");
    println!("{}", vibrant.border().to_string().repeat(60));
    println!("{}", pastel.border().to_string().repeat(60));
    println!("{}", minimal.border().to_string().repeat(60));
    println!();

    // Colorized text
    println!("=== Colorized Text ===");
    println!("{}", vibrant.colorize("Hello from Vibrant scheme!"));
    println!("{}", pastel.colorize("Hello from Pastel scheme!"));
    println!("{}", minimal.colorize("Hello from Minimal scheme!"));
}