use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use rand::Rng;
use crossterm::{
    cursor, execute, queue, style,
    terminal::{self, ClearType},
    event::{self, Event, KeyCode},
};

const NUM_STARS: usize = 50;
const NUM_DROPS: usize = 30;
const NUM_PARTICLES: usize = 20;
const NUM_MATRIX_COLS: usize = 20;

struct Star {
    x: u16,
    y: u16,
    speed: u16,
    brightness: u8,
}

struct Drop {
    x: u16,
    y: u16,
    speed: u16,
}

struct Particle {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    color: style::Color,
}

struct MatrixColumn {
    x: u16,
    y: u16,
    length: u16,
    speed: u16,
}

fn main() {
    loop {
        match show_menu() {
            Some(choice) => {
                if choice == "8" || choice == "exit" {
                    cleanup();
                    break;
                }
                
                if choice == "9" || choice == "all" {
                    run_all_animations();
                } else if choice == "10" || choice == "hybrid" {
                    show_hybrid_intro();
                    run_hybrid_animation();
                } else if choice.contains('-') {
                    run_multiple_animations(&choice);
                } else if let Ok(num) = choice.parse::<u8>() {
                    run_animation(num);
                } else {
                    println!("Invalid choice");
                    thread::sleep(Duration::from_secs(1));
                }
            }
            None => break,
        }
    }
}

fn show_menu() -> Option<String> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).ok();
    
    println!("╔════════════════════════════════════════╗");
    println!("║  TERMINAL BACKGROUND ANIMATIONS        ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    println!("  1) Matrix Rain");
    println!("  2) Starfield");
    println!("  3) Wave Animation");
    println!("  4) Rain Effect");
    println!("  5) Gradient Background");
    println!("  6) Particle System");
    println!("  7) Digital Clock");
    println!("  8) Exit");
    println!("  9) Run All Animations (Sequential)");
    println!("  10) 🎨 HYBRID MODE - All at Once! 🎨");
    println!();
    println!("  Multi-select: Enter numbers with hyphens (e.g., 1-3-5)");
    println!();
    print!("Choose animation (1-10): ");
    stdout.flush().ok();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    Some(input.trim().to_string())
}

fn show_hybrid_intro() {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).ok();
    
    println!("\x1b[35m"); // Magenta
    println!("╔═══════════════════════════════════════════════╗");
    println!("║     🎨 HYBRID ANIMATION MODE 🎨              ║");
    println!("║                                               ║");
    println!("║  All 7 animations running simultaneously!    ║");
    println!("║                                               ║");
    println!("║  Layers:                                      ║");
    println!("║    1. Gradient Background                     ║");
    println!("║    2. Wave Animation                          ║");
    println!("║    3. Starfield                               ║");
    println!("║    4. Matrix Rain                             ║");
    println!("║    5. Rain Effect                             ║");
    println!("║    6. Particle System                         ║");
    println!("║    7. Digital Clock                           ║");
    println!("║                                               ║");
    println!("║  Press ESC or Q to exit                       ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!("\x1b[0m");
    println!();
    
    for i in (1..=3).rev() {
        print!("\x1b[33mStarting in {}...\x1b[0m", i);
        stdout.flush().ok();
        thread::sleep(Duration::from_secs(1));
        print!("\r                    \r");
        stdout.flush().ok();
    }
    
    println!("\x1b[32mGO! 🚀\x1b[0m");
    thread::sleep(Duration::from_millis(500));
}

fn run_hybrid_animation() {
    let mut stdout = io::stdout();
    
    // Enter alternate screen and setup terminal
    terminal::enable_raw_mode().ok();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::Hide
    ).ok();
    
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    // Initialize all animation components
    let mut stars: Vec<Star> = (0..NUM_STARS)
        .map(|_| Star {
            x: rng.random_range(0..cols),
            y: rng.random_range(0..rows),
            speed: rng.random_range(1..3),
            brightness: rng.random_range(0..3),
        })
        .collect();
    
    let mut drops: Vec<Drop> = (0..NUM_DROPS)
        .map(|_| Drop {
            x: rng.random_range(0..cols),
            y: rng.random_range(0..rows),
            speed: rng.random_range(2..4),
        })
        .collect();
    
    let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| {
            let colors = [
                style::Color::Red, style::Color::Green, style::Color::Yellow,
                style::Color::Blue, style::Color::Magenta, style::Color::Cyan,
            ];
            Particle {
                x: rng.random_range(0..cols as i32),
                y: rng.random_range(0..rows as i32),
                vx: rng.random_range(-1..=1),
                vy: rng.random_range(-1..=1),
                color: colors[rng.random_range(0..colors.len())],
            }
        })
        .collect();
    
    let mut matrix_cols: Vec<MatrixColumn> = (0..NUM_MATRIX_COLS)
        .map(|_| MatrixColumn {
            x: rng.random_range(0..cols),
            y: rng.random_range(0..rows),
            length: rng.random_range(5..20),
            speed: rng.random_range(1..4),
        })
        .collect();
    
    let mut frame = 0;
    let mut gradient_shift = 0;
    let mut wave_offset = 0.0;
    
    loop {
        if check_exit_key() {
            break;
        }
        
        // Clear screen for this frame
        queue!(stdout, cursor::MoveTo(0, 0)).ok();
        
        // Layer 1: Gradient Background (update every 3 frames)
        if frame % 3 == 0 {
            draw_gradient_layer(&mut stdout, cols, rows, gradient_shift);
            gradient_shift += 1;
        }
        
        // Layer 2: Wave Animation (update every 2 frames)
        if frame % 2 == 0 {
            draw_wave_layer(&mut stdout, cols, rows, wave_offset);
            wave_offset += 1.0;
        }
        
        // Layer 3: Starfield
        draw_starfield_layer(&mut stdout, &mut stars, cols, rows, &mut rng);
        
        // Layer 4: Matrix Rain (update every 2 frames)
        if frame % 2 == 0 {
            draw_matrix_layer(&mut stdout, &mut matrix_cols, cols, rows, &mut rng);
        }
        
        // Layer 5: Rain Effect
        draw_rain_layer(&mut stdout, &mut drops, cols, rows, &mut rng);
        
        // Layer 6: Particle System
        draw_particles_layer(&mut stdout, &mut particles, cols, rows, &mut rng);
        
        // Layer 7: Digital Clock (always on top)
        draw_clock_layer(&mut stdout, cols, rows);
        
        stdout.flush().ok();
        frame += 1;
        thread::sleep(Duration::from_millis(50)); // ~20 FPS
    }
    
    // Exit alternate screen and cleanup
    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    ).ok();
    terminal::disable_raw_mode().ok();
    
    println!("\n\x1b[36m✨ Animation stopped. Thanks for watching!\x1b[0m\n");
}

fn draw_gradient_layer(stdout: &mut io::Stdout, cols: u16, rows: u16, shift: i32) {
    let start_row = rows / 4;
    let end_row = (rows * 3) / 4;
    
    for y in start_row..end_row {
        let intensity = ((y as i32 - start_row as i32 + shift) * 51 / (end_row - start_row) as i32) % 52;
        let color = 232 + intensity as u8;
        
        queue!(
            stdout,
            cursor::MoveTo(0, y),
            style::SetBackgroundColor(style::Color::AnsiValue(color))
        ).ok();
        
        for _ in 0..cols {
            queue!(stdout, style::Print(' ')).ok();
        }
        queue!(stdout, style::ResetColor).ok();
    }
}

fn draw_wave_layer(stdout: &mut io::Stdout, cols: u16, rows: u16, offset: f64) {
    let wave_row = rows / 2;
    
    for x in (0..cols).step_by(2) {
        let angle = (x as f64 + offset) * 0.2;
        let wave = (angle.sin() * 8.0) as i32;
        let y = (wave_row as i32 + wave) as u16;
        
        if y < rows {
            queue!(
                stdout,
                cursor::MoveTo(x, y),
                style::SetForegroundColor(style::Color::Rgb { r: 0, g: 191, b: 255 }),
                style::Print('≈'),
                style::ResetColor
            ).ok();
        }
    }
}

fn draw_starfield_layer(stdout: &mut io::Stdout, stars: &mut Vec<Star>, cols: u16, rows: u16, rng: &mut rand::rngs::ThreadRng) {
    for star in stars.iter_mut() {
        if star.x < cols && star.y < rows {
            let (ch, color) = match star.brightness {
                0 => ('.', style::Color::DarkGrey),
                1 => ('*', style::Color::White),
                _ => ('✦', style::Color::Rgb { r: 255, g: 255, b: 255 }),
            };
            
            queue!(
                stdout,
                cursor::MoveTo(star.x, star.y),
                style::SetForegroundColor(color),
                style::Print(ch),
                style::ResetColor
            ).ok();
        }
        
        if star.x >= star.speed {
            star.x -= star.speed;
        } else {
            star.x = cols - 1;
            star.y = rng.random_range(0..rows);
            star.brightness = rng.random_range(0..3);
        }
    }
}

fn draw_matrix_layer(stdout: &mut io::Stdout, matrix_cols: &mut Vec<MatrixColumn>, cols: u16, rows: u16, rng: &mut rand::rngs::ThreadRng) {
    let matrix_chars = ['0', '1', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク'];
    
    for col in matrix_cols.iter_mut() {
        for j in 0..col.length {
            let row = (col.y + j) % rows;
            
            if col.x < cols && row < rows {
                let ch = matrix_chars[rng.random_range(0..matrix_chars.len())];
                let color = if j == 0 {
                    style::Color::Rgb { r: 150, g: 255, b: 150 } // Bright head
                } else if j < 3 {
                    style::Color::Green // Medium
                } else {
                    style::Color::DarkGreen // Dim tail
                };
                
                queue!(
                    stdout,
                    cursor::MoveTo(col.x, row),
                    style::SetForegroundColor(color),
                    style::Print(ch),
                    style::ResetColor
                ).ok();
            }
        }
        
        col.y = (col.y + col.speed) % rows;
        
        // Occasionally respawn column
        if rng.random_range(0..20) == 0 {
            col.x = rng.random_range(0..cols);
            col.length = rng.random_range(5..20);
            col.y = 0;
        }
    }
}

fn draw_rain_layer(stdout: &mut io::Stdout, drops: &mut Vec<Drop>, cols: u16, rows: u16, rng: &mut rand::rngs::ThreadRng) {
    for drop in drops.iter_mut() {
        if drop.x < cols && drop.y < rows {
            queue!(
                stdout,
                cursor::MoveTo(drop.x, drop.y),
                style::SetForegroundColor(style::Color::Rgb { r: 51, g: 255, b: 255 }),
                style::Print('│'),
                style::ResetColor
            ).ok();
        }
        
        drop.y += drop.speed;
        if drop.y >= rows {
            drop.y = 0;
            drop.x = rng.random_range(0..cols);
        }
    }
}

fn draw_particles_layer(stdout: &mut io::Stdout, particles: &mut Vec<Particle>, cols: u16, rows: u16, rng: &mut rand::rngs::ThreadRng) {
    let colors = [
        style::Color::Red, style::Color::Green, style::Color::Yellow,
        style::Color::Blue, style::Color::Magenta, style::Color::Cyan,
    ];
    
    for p in particles.iter_mut() {
        if p.x >= 0 && p.x < cols as i32 && p.y >= 0 && p.y < rows as i32 {
            queue!(
                stdout,
                cursor::MoveTo(p.x as u16, p.y as u16),
                style::SetForegroundColor(p.color),
                style::Print('●'),
                style::ResetColor
            ).ok();
        }
        
        p.x += p.vx;
        p.y += p.vy;
        
        // Bounce off edges
        if p.x <= 0 { p.vx = 1; }
        if p.x >= cols as i32 - 1 { p.vx = -1; }
        if p.y <= 0 { p.vy = 1; }
        if p.y >= rows as i32 - 1 { p.vy = -1; }
        
        // Randomly change color occasionally
        if rng.random_range(0..50) == 0 {
            p.color = colors[rng.random_range(0..colors.len())];
        }
    }
}

fn draw_clock_layer(stdout: &mut io::Stdout, cols: u16, rows: u16) {
    let now = chrono::Local::now();
    let time_str = now.format("%H:%M:%S").to_string();
    let date_str = now.format("%Y-%m-%d").to_string();
    
    let x = (cols / 2).saturating_sub(8);
    let y = 2;
    
    // Draw clock border
    queue!(
        stdout,
        cursor::MoveTo(x, y - 1),
        style::SetForegroundColor(style::Color::Yellow),
        style::Print("╔══════════════╗"),
        style::ResetColor
    ).ok();
    
    queue!(
        stdout,
        cursor::MoveTo(x, y),
        style::SetForegroundColor(style::Color::Yellow),
        style::Print("║ "),
        style::SetForegroundColor(style::Color::Cyan),
        style::Print(&time_str),
        style::SetForegroundColor(style::Color::Yellow),
        style::Print(" ║"),
        style::ResetColor
    ).ok();
    
    queue!(
        stdout,
        cursor::MoveTo(x, y + 1),
        style::SetForegroundColor(style::Color::Yellow),
        style::Print("╚══════════════╝"),
        style::ResetColor
    ).ok();
    
    // Draw date
    let date_x = (cols / 2).saturating_sub((date_str.len() / 2) as u16);
    queue!(
        stdout,
        cursor::MoveTo(date_x, y + 2),
        style::SetForegroundColor(style::Color::Rgb { r: 255, g: 165, b: 0 }),
        style::Print(&date_str),
        style::ResetColor
    ).ok();
    
    // Draw info
    let info = "Press ESC or Q to exit | ~20 FPS";
    let info_x = (cols / 2).saturating_sub((info.len() / 2) as u16);
    if rows > 4 {
        queue!(
            stdout,
            cursor::MoveTo(info_x, rows - 2),
            style::SetForegroundColor(style::Color::DarkGrey),
            style::Print(info),
            style::ResetColor
        ).ok();
    }
}

fn run_all_animations() {
    for i in 1..=7 {
        if check_exit_key() {
            break;
        }
        run_animation_timed(i, Duration::from_secs(5));
    }
}

fn run_multiple_animations(choice: &str) {
    let selections: Vec<u8> = choice
        .split('-')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    
    for num in selections {
        if num >= 1 && num <= 7 {
            if check_exit_key() {
                break;
            }
            run_animation_timed(num, Duration::from_secs(5));
        }
    }
}

fn run_animation(num: u8) {
    setup_terminal();
    
    match num {
        1 => matrix_rain(),
        2 => starfield(),
        3 => wave_animation(),
        4 => rain_effect(),
        5 => gradient_bg(),
        6 => particles(),
        7 => digital_clock(),
        _ => {}
    }
    
    cleanup();
}

fn run_animation_timed(num: u8, duration: Duration) {
    setup_terminal();
    
    let start = std::time::Instant::now();
    
    match num {
        1 => matrix_rain_timed(start, duration),
        2 => starfield_timed(start, duration),
        3 => wave_animation_timed(start, duration),
        4 => rain_effect_timed(start, duration),
        5 => gradient_bg_timed(start, duration),
        6 => particles_timed(start, duration),
        7 => digital_clock_timed(start, duration),
        _ => {}
    }
    
    cleanup();
}

fn setup_terminal() {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::Hide
    ).ok();
    terminal::enable_raw_mode().ok();
}

fn cleanup() {
    let mut stdout = io::stdout();
    terminal::disable_raw_mode().ok();
    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    ).ok();
    
    println!("\n\x1b[36m✨ Animation stopped. Thanks for watching!\x1b[0m\n");
}

fn check_exit_key() -> bool {
    if event::poll(Duration::from_millis(0)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            return matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q'));
        }
    }
    false
}

fn should_continue(start: std::time::Instant, duration: Duration) -> bool {
    start.elapsed() < duration && !check_exit_key()
}

// Individual animation implementations...

fn matrix_rain() {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    loop {
        if check_exit_key() { break; }
        
        let col = rng.random_range(0..cols);
        let row = rng.random_range(0..rows);
        let ch = char::from_u32(rng.random_range(33..127)).unwrap_or('*');
        let color = if rng.random_bool(0.5) { style::Color::Green } else { style::Color::DarkGreen };
        
        queue!(
            stdout,
            cursor::MoveTo(col, row),
            style::SetForegroundColor(color),
            style::Print(ch),
            style::ResetColor
        ).ok();
        stdout.flush().ok();
        
        thread::sleep(Duration::from_millis(50));
    }
}

fn matrix_rain_timed(start: std::time::Instant, duration: Duration) {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    while should_continue(start, duration) {
        let col = rng.random_range(0..cols);
        let row = rng.random_range(0..rows);
        let ch = char::from_u32(rng.random_range(33..127)).unwrap_or('*');
        let color = if rng.random_bool(0.5) { style::Color::Green } else { style::Color::DarkGreen };
        
        queue!(
            stdout,
            cursor::MoveTo(col, row),
            style::SetForegroundColor(color),
            style::Print(ch),
            style::ResetColor
        ).ok();
        stdout.flush().ok();
        
        thread::sleep(Duration::from_millis(50));
    }
}

fn starfield() {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    let mut stars: Vec<Star> = (0..NUM_STARS)
        .map(|_| Star {
            x: rng.random_range(0..cols),
            y: rng.random_range(0..rows),
            speed: rng.random_range(1..4),
            brightness: rng.random_range(0..3),
        })
        .collect();
    
    loop {
        if check_exit_key() { break; }
        
        for star in &mut stars {
            queue!(stdout, cursor::MoveTo(star.x, star.y), style::Print(' ')).ok();
            
            star.x += star.speed;
            if star.x >= cols {
                star.x = 0;
                star.y = rng.random_range(0..rows);
            }
            
            queue!(
                stdout,
                cursor::MoveTo(star.x, star.y),
                style::SetForegroundColor(style::Color::White),
                style::Print('*'),
                style::ResetColor
            ).ok();
        }
        stdout.flush().ok();
        thread::sleep(Duration::from_millis(100));
    }
}

fn starfield_timed(start: std::time::Instant, duration: Duration) {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    let mut stars: Vec<Star> = (0..NUM_STARS)
        .map(|_| Star {
            x: rng.random_range(0..cols),
            y: rng.random_range(0..rows),
            speed: rng.random_range(1..4),
            brightness: rng.random_range(0..3),
        })
        .collect();
    
    while should_continue(start, duration) {
        for star in &mut stars {
            queue!(stdout, cursor::MoveTo(star.x, star.y), style::Print(' ')).ok();
            
            star.x += star.speed;
            if star.x >= cols {
                star.x = 0;
                star.y = rng.random_range(0..rows);
            }
            
            queue!(
                stdout,
                cursor::MoveTo(star.x, star.y),
                style::SetForegroundColor(style::Color::White),
                style::Print('*'),
                style::ResetColor
            ).ok();
        }
        stdout.flush().ok();
        thread::sleep(Duration::from_millis(100));
    }
}

fn wave_animation() {
    let mut stdout = io::stdout();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let mut offset = 0.0;
    
    loop {
        if check_exit_key() { break; }
        
        execute!(stdout, terminal::Clear(ClearType::All)).ok();
        
        for y in 0..rows {
            for x in 0..cols {
                let wave = ((x as f64 + offset) * 0.1).sin() * 5.0 + y as f64;
                if (wave as u16) == y {
                    queue!(
                        stdout,
                        cursor::MoveTo(x, y),
                        style::SetForegroundColor(style::Color::Blue),
                        style::Print('~'),
                        style::ResetColor
                    ).ok();
                }
            }
        }
        stdout.flush().ok();
        offset += 1.0;
        thread::sleep(Duration::from_millis(50));
    }
}

fn wave_animation_timed(start: std::time::Instant, duration: Duration) {
    let mut stdout = io::stdout();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let mut offset = 0.0;
    
    while should_continue(start, duration) {
        execute!(stdout, terminal::Clear(ClearType::All)).ok();
        
        for y in 0..rows {
            for x in 0..cols {
                let wave = ((x as f64 + offset) * 0.1).sin() * 5.0 + y as f64;
                if (wave as u16) == y {
                    queue!(
                        stdout,
                        cursor::MoveTo(x, y),
                        style::SetForegroundColor(style::Color::Blue),
                        style::Print('~'),
                        style::ResetColor
                    ).ok();
                }
            }
        }
        stdout.flush().ok();
        offset += 1.0;
        thread::sleep(Duration::from_millis(50));
    }
}

fn rain_effect() {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    let mut drops: Vec<Drop> = (0..NUM_DROPS)
        .map(|_| Drop {
            x: rng.random_range(0..cols),
            y: rng.random_range(0..rows),
            speed: rng.random_range(2..4),
        })
        .collect();
    
    loop {
        if check_exit_key() { break; }
        
        for drop in &mut drops {
            queue!(stdout, cursor::MoveTo(drop.x, drop.y), style::Print(' ')).ok();
            
            drop.y += 1;
            if drop.y >= rows {
                drop.y = 0;
                drop.x = rng.random_range(0..cols);
            }
            
            queue!(
                stdout,
                cursor::MoveTo(drop.x, drop.y),
                style::SetForegroundColor(style::Color::Cyan),
                style::Print('|'),
                style::ResetColor
            ).ok();
        }
        stdout.flush().ok();
        thread::sleep(Duration::from_millis(80));
    }
}

fn rain_effect_timed(start: std::time::Instant, duration: Duration) {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    let mut drops: Vec<Drop> = (0..NUM_DROPS)
        .map(|_| Drop {
            x: rng.random_range(0..cols),
            y: rng.random_range(0..rows),
            speed: rng.random_range(2..4),
        })
        .collect();
    
    while should_continue(start, duration) {
        for drop in &mut drops {
            queue!(stdout, cursor::MoveTo(drop.x, drop.y), style::Print(' ')).ok();
            
            drop.y += 1;
            if drop.y >= rows {
                drop.y = 0;
                drop.x = rng.random_range(0..cols);
            }
            
            queue!(
                stdout,
                cursor::MoveTo(drop.x, drop.y),
                style::SetForegroundColor(style::Color::Cyan),
                style::Print('|'),
                style::ResetColor
            ).ok();
        }
        stdout.flush().ok();
        thread::sleep(Duration::from_millis(80));
    }
}

fn gradient_bg() {
    let mut stdout = io::stdout();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let mut shift = 0;
    
    loop {
        if check_exit_key() { break; }
        
        for y in 0..rows {
            let color_idx = 16 + ((y as i32 + shift) % 216);
            queue!(
                stdout,
                cursor::MoveTo(0, y),
                style::SetBackgroundColor(style::Color::AnsiValue(color_idx as u8))
            ).ok();
            
            for _ in 0..cols {
                queue!(stdout, style::Print(' ')).ok();
            }
            queue!(stdout, style::ResetColor).ok();
        }
        stdout.flush().ok();
        shift += 1;
        thread::sleep(Duration::from_millis(100));
    }
}

fn gradient_bg_timed(start: std::time::Instant, duration: Duration) {
    let mut stdout = io::stdout();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let mut shift = 0;
    
    while should_continue(start, duration) {
        for y in 0..rows {
            let color_idx = 16 + ((y as i32 + shift) % 216);
            queue!(
                stdout,
                cursor::MoveTo(0, y),
                style::SetBackgroundColor(style::Color::AnsiValue(color_idx as u8))
            ).ok();
            
            for _ in 0..cols {
                queue!(stdout, style::Print(' ')).ok();
            }
            queue!(stdout, style::ResetColor).ok();
        }
        stdout.flush().ok();
        shift += 1;
        thread::sleep(Duration::from_millis(100));
    }
}

fn particles() {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| {
            let colors = [
                style::Color::Red, style::Color::Green, style::Color::Yellow,
                style::Color::Blue, style::Color::Magenta, style::Color::Cyan,
            ];
            Particle {
                x: rng.random_range(0..cols as i32),
                y: rng.random_range(0..rows as i32),
                vx: rng.random_range(-1..=1),
                vy: rng.random_range(-1..=1),
                color: colors[rng.random_range(0..colors.len())],
            }
        })
        .collect();
    
    loop {
        if check_exit_key() { break; }
        
        for p in &mut particles {
            if p.x >= 0 && p.x < cols as i32 && p.y >= 0 && p.y < rows as i32 {
                queue!(stdout, cursor::MoveTo(p.x as u16, p.y as u16), style::Print(' ')).ok();
            }
            
            p.x += p.vx;
            p.y += p.vy;
            
            if p.x <= 0 { p.vx = 1; }
            if p.x >= cols as i32 - 1 { p.vx = -1; }
            if p.y <= 0 { p.vy = 1; }
            if p.y >= rows as i32 - 1 { p.vy = -1; }
            
            if p.x >= 0 && p.x < cols as i32 && p.y >= 0 && p.y < rows as i32 {
                queue!(
                    stdout,
                    cursor::MoveTo(p.x as u16, p.y as u16),
                    style::SetForegroundColor(p.color),
                    style::Print('●'),
                    style::ResetColor
                ).ok();
            }
        }
        stdout.flush().ok();
        thread::sleep(Duration::from_millis(100));
    }
}

fn particles_timed(start: std::time::Instant, duration: Duration) {
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    
    let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| {
            let colors = [
                style::Color::Red, style::Color::Green, style::Color::Yellow,
                style::Color::Blue, style::Color::Magenta, style::Color::Cyan,
            ];
            Particle {
                x: rng.random_range(0..cols as i32),
                y: rng.random_range(0..rows as i32),
                vx: rng.random_range(-1..=1),
                vy: rng.random_range(-1..=1),
                color: colors[rng.random_range(0..colors.len())],
            }
        })
        .collect();
    
    while should_continue(start, duration) {
        for p in &mut particles {
            if p.x >= 0 && p.x < cols as i32 && p.y >= 0 && p.y < rows as i32 {
                queue!(stdout, cursor::MoveTo(p.x as u16, p.y as u16), style::Print(' ')).ok();
            }
            
            p.x += p.vx;
            p.y += p.vy;
            
            if p.x <= 0 { p.vx = 1; }
            if p.x >= cols as i32 - 1 { p.vx = -1; }
            if p.y <= 0 { p.vy = 1; }
            if p.y >= rows as i32 - 1 { p.vy = -1; }
            
            if p.x >= 0 && p.x < cols as i32 && p.y >= 0 && p.y < rows as i32 {
                queue!(
                    stdout,
                    cursor::MoveTo(p.x as u16, p.y as u16),
                    style::SetForegroundColor(p.color),
                    style::Print('●'),
                    style::ResetColor
                ).ok();
            }
        }
        stdout.flush().ok();
        thread::sleep(Duration::from_millis(100));
    }
}

fn digital_clock() {
    let mut stdout = io::stdout();
    
    loop {
        if check_exit_key() { break; }
        
        let now = chrono::Local::now();
        let time_str = now.format("%H:%M:%S").to_string();
        
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let x = (cols / 2).saturating_sub((time_str.len() / 2) as u16);
        let y = rows / 2;
        
        execute!(stdout, terminal::Clear(ClearType::All)).ok();
        queue!(
            stdout,
            cursor::MoveTo(x, y),
            style::SetForegroundColor(style::Color::Green),
            style::Print(&time_str),
            style::ResetColor
        ).ok();
        stdout.flush().ok();
        
        thread::sleep(Duration::from_secs(1));
    }
}

fn digital_clock_timed(start: std::time::Instant, duration: Duration) {
    let mut stdout = io::stdout();
    
    while should_continue(start, duration) {
        let now = chrono::Local::now();
        let time_str = now.format("%H:%M:%S").to_string();
        
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let x = (cols / 2).saturating_sub((time_str.len() / 2) as u16);
        let y = rows / 2;
        
        execute!(stdout, terminal::Clear(ClearType::All)).ok();
        queue!(
            stdout,
            cursor::MoveTo(x, y),
            style::SetForegroundColor(style::Color::Green),
            style::Print(&time_str),
            style::ResetColor
        ).ok();
        stdout.flush().ok();
        
        thread::sleep(Duration::from_secs(1));
    }
}