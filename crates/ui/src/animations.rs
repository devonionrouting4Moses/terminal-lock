use crossterm::{
    cursor::{MoveTo, Show},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{size, Clear, ClearType},
};
use rand::Rng;
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

/// Animation manager for background effects
pub struct AnimationManager {
    running: bool,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self { running: false }
    }

    /// Start matrix rain animation in background (full terminal)
    pub fn start_matrix(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        thread::spawn(move || {
            let mut stdout = stdout();
            // Don't hide cursor - let it show for terminal usage
            // execute!(stdout, Hide).ok();

            let (cols, rows) = size().unwrap_or((80, 24));
            let mut rng = rand::rng();

            // Matrix column states (full terminal width)
            let mut columns: Vec<(u16, u16)> = (0..20)
                .map(|_| (
                    rng.random_range(0..cols),
                    rng.random_range(0..rows)
                ))
                .collect();

            loop {
                for (col, row) in columns.iter_mut() {
                    // Draw character with lower opacity effect
                    let chars = ['0', '1', 'ア', 'イ', 'ウ', 'エ', 'オ'];
                    let ch = chars[rng.random_range(0..chars.len())];

                    execute!(
                        stdout,
                        MoveTo(*col, *row),
                        SetForegroundColor(Color::Green),
                        Print(ch)
                    )
                    .ok();

                    // Move down across full terminal height
                    *row = if *row >= rows.saturating_sub(1) {
                        0
                    } else {
                        *row + 1
                    };

                    // Randomly reset column across full width
                    if rng.random_bool(0.02) {
                        *col = rng.random_range(0..cols);
                        *row = 0;
                    }
                }

                stdout.flush().ok();
                thread::sleep(Duration::from_millis(150));
            }
        });
    }

    /// Start starfield animation
    pub fn start_starfield(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        thread::spawn(move || {
            let mut stdout = stdout();
            // Don't hide cursor - let it show for terminal usage
            // execute!(stdout, Hide).ok();

            let (cols, rows) = size().unwrap_or((80, 24));
            let mut rng = rand::rng();

            // Star states: (x, y, speed, brightness)
            let mut stars: Vec<(u16, u16, u16, u8)> = (0..25)
                .map(|_| {
                    (
                        rng.random_range(0..cols),
                        rng.random_range(0..rows),
                        rng.random_range(1..4),
                        rng.random_range(0..3),
                    )
                })
                .collect();

            loop {
                for (x, y, speed, brightness) in stars.iter_mut() {
                    // Clear old position
                    execute!(stdout, MoveTo(*x, *y), Print(' ')).ok();

                    // Move star
                    *x = if *x >= *speed { *x - *speed } else { cols };

                    // Reset if off screen
                    if *x == 0 || *x >= cols {
                        *x = cols - 1;
                        *y = rng.random_range(0..rows);
                        *brightness = rng.random_range(0..3);
                    }

                    // Draw star with brightness
                    let (color, ch) = match *brightness {
                        0 => (Color::DarkGrey, '.'),
                        1 => (Color::White, '*'),
                        _ => (Color::Yellow, '✦'),
                    };

                    execute!(
                        stdout,
                        MoveTo(*x, *y),
                        SetForegroundColor(color),
                        Print(ch)
                    )
                    .ok();
                }

                stdout.flush().ok();
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    /// Start particle system animation
    pub fn start_particles(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        thread::spawn(move || {
            let mut stdout = stdout();
            // Don't hide cursor - let it show for terminal usage
            // execute!(stdout, Hide).ok();

            let (cols, rows) = size().unwrap_or((80, 24));
            let mut rng = rand::rng();

            // Particle states: (x, y, vx, vy, color)
            let mut particles: Vec<(i16, i16, i16, i16, Color)> = (0..18)
                .map(|_| {
                    let colors = [
                        Color::Red,
                        Color::Green,
                        Color::Blue,
                        Color::Yellow,
                        Color::Magenta,
                        Color::Cyan,
                    ];
                    (
                        rng.random_range(0..cols as i16),
                        rng.random_range(0..rows as i16),
                        rng.random_range(-1..=1),
                        rng.random_range(-1..=1),
                        colors[rng.random_range(0..colors.len())],
                    )
                })
                .collect();

            loop {
                for (x, y, vx, vy, color) in particles.iter_mut() {
                    // Clear old position
                    if *x >= 0 && *x < cols as i16 && *y >= 0 && *y < rows as i16 {
                        execute!(stdout, MoveTo(*x as u16, *y as u16), Print(' ')).ok();
                    }

                    // Update position
                    *x += *vx;
                    *y += *vy;

                    // Bounce off edges
                    if *x <= 0 || *x >= cols as i16 - 1 {
                        *vx = -*vx;
                    }
                    if *y <= 0 || *y >= rows as i16 - 1 {
                        *vy = -*vy;
                    }

                    // Clamp position
                    *x = (*x).clamp(0, cols as i16 - 1);
                    *y = (*y).clamp(0, rows as i16 - 1);

                    // Draw particle
                    execute!(
                        stdout,
                        MoveTo(*x as u16, *y as u16),
                        SetForegroundColor(*color),
                        Print('●')
                    )
                    .ok();
                }

                stdout.flush().ok();
                thread::sleep(Duration::from_millis(150));
            }
        });
    }

    /// Stop animations and clean up
    pub fn stop(&mut self) {
        self.running = false;
        let mut stdout = stdout();
        execute!(stdout, Show, Clear(ClearType::All)).ok();
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AnimationManager {
    fn drop(&mut self) {
        // DO NOT stop animations on drop
        // Animations should continue running in background thread indefinitely
        // This prevents the screen from clearing when AnimationManager is dropped
    }
}