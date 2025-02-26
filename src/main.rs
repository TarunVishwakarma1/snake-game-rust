use piston_window::*;
use rand::prelude::*;
use rand::rng;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::{DateTime, Local};

const GRID_SIZE: u32 = 20;
const CELL_SIZE: u32 = 25;
const WIDTH: u32 = GRID_SIZE * CELL_SIZE;
const HEIGHT: u32 = GRID_SIZE * CELL_SIZE;

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: u32,
    y: u32,
}

struct GameStats {
    start_time: SystemTime,
    time_played: Duration,
    up_turns: u32,
    down_turns: u32,
    left_turns: u32,
    right_turns: u32,
    food_eaten: u32,
    timestamp: u64,
}

impl GameStats {
    fn new() -> Self {
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
            
        Self {
            start_time: now,
            time_played: Duration::from_secs(0),
            up_turns: 0,
            down_turns: 0,
            left_turns: 0,
            right_turns: 0,
            food_eaten: 0,
            timestamp,
        }
    }
    
    fn update(&mut self) {
        self.time_played = SystemTime::now().duration_since(self.start_time).unwrap_or(Duration::from_secs(0));
    }
    
    fn save_to_file(&self, final_score: u32) -> std::io::Result<()> {
        // Format timestamp as human-readable date/time for filename
        let dt: DateTime<Local> = self.start_time.into();
        let filename = format!("{}_snake_game_stats.txt", dt.format("%Y%m%d_%H%M%S"));
        
        let mut file = File::create(filename)?;
        
        // Convert times to more readable format
        let time_played_secs = self.time_played.as_secs();
        let minutes = time_played_secs / 60;
        let seconds = time_played_secs % 60;
        
        // Write stats to file
        writeln!(file, "Snake Game Statistics")?;
        writeln!(file, "=====================")?;
        writeln!(file, "Game started at: {}", dt.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file, "Time played: {}m {}s", minutes, seconds)?;
        writeln!(file, "Final score: {}", final_score)?;
        writeln!(file, "Food eaten: {}", self.food_eaten)?;
        writeln!(file, "")?;
        writeln!(file, "Movement Statistics:")?;
        writeln!(file, "  Up turns: {}", self.up_turns)?;
        writeln!(file, "  Down turns: {}", self.down_turns)?;
        writeln!(file, "  Left turns: {}", self.left_turns)?;
        writeln!(file, "  Right turns: {}", self.right_turns)?;
        writeln!(file, "")?;
        writeln!(file, "Total turns: {}", self.up_turns + self.down_turns + self.left_turns + self.right_turns)?;
        
        Ok(())
    }
}

struct Game {
    snake: Vec<Position>,
    food: Position,
    direction: Direction,
    is_game_over: bool,
    score: u32,
    speed: f64,
    last_update: f64,
    stats: GameStats,
}

impl Game {
    fn new() -> Game {
        let mut rng = rng();
        
        Game {
            snake: vec![Position { x: 10, y: 10 }],
            food: Position {
                x: rng.random_range(0..GRID_SIZE),
                y: rng.random_range(0..GRID_SIZE),
            },
            direction: Direction::Right,
            is_game_over: false,
            score: 0,
            speed: 0.1, // Time between updates in seconds
            last_update: 0.0,
            stats: GameStats::new(),
        }
    }

    fn update(&mut self, dt: f64) -> bool {
        if self.is_game_over {
            return false;
        }

        // Update stats time played
        self.stats.update();

        self.last_update += dt;
        if self.last_update < self.speed {
            return false;
        }
        self.last_update = 0.0;

        let head = self.snake[0];
        let mut new_head = head;

        match self.direction {
            Direction::Up => new_head.y = (new_head.y + GRID_SIZE - 1) % GRID_SIZE,
            Direction::Down => new_head.y = (new_head.y + 1) % GRID_SIZE,
            Direction::Left => new_head.x = (new_head.x + GRID_SIZE - 1) % GRID_SIZE,
            Direction::Right => new_head.x = (new_head.x + 1) % GRID_SIZE,
        }

        // Check collision with self
        if self.snake.iter().skip(1).any(|p| p.x == new_head.x && p.y == new_head.y) {
            self.is_game_over = true;
            // Save stats to file when game is over
            if let Err(e) = self.stats.save_to_file(self.score) {
                eprintln!("Error saving stats: {}", e);
            }
            return true;
        }

        self.snake.insert(0, new_head);

        if new_head.x == self.food.x && new_head.y == self.food.y {
            // Ate food, grow snake and spawn new food
            self.score += 1;
            self.stats.food_eaten += 1;
            self.speed = (0.1 - (self.score as f64 * 0.002)).max(0.05); // Speed up as score increases
            self.spawn_food();
        } else {
            // Remove tail if no food was eaten
            self.snake.pop();
        }

        true
    }

    fn spawn_food(&mut self) {
        let mut rng = rng();
        let mut new_food;

        loop {
            new_food = Position {
                x: rng.random_range(0..GRID_SIZE),
                y: rng.random_range(0..GRID_SIZE),
            };

            // Make sure food doesn't spawn on snake
            if !self.snake.iter().any(|p| p.x == new_food.x && p.y == new_food.y) {
                break;
            }
        }

        self.food = new_food;
    }

    fn change_direction(&mut self, new_direction: Direction) {
        if new_direction.opposite() != self.direction {
            // Update turn statistics
            match new_direction {
                Direction::Up => self.stats.up_turns += 1,
                Direction::Down => self.stats.down_turns += 1,
                Direction::Left => self.stats.left_turns += 1,
                Direction::Right => self.stats.right_turns += 1,
            }
            
            self.direction = new_direction;
        }
    }

    fn reset(&mut self) {
        // Save stats of the previous game
        if let Err(e) = self.stats.save_to_file(self.score) {
            eprintln!("Error saving stats: {}", e);
        }
        
        // Create a new game
        *self = Game::new();
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Snake Game", [WIDTH, HEIGHT])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game::new();
    let mut events = Events::new(EventSettings::new().ups(60));

    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Up => game.change_direction(Direction::Up),
                Key::Down => game.change_direction(Direction::Down),
                Key::Left => game.change_direction(Direction::Left),
                Key::Right => game.change_direction(Direction::Right),
                Key::R if game.is_game_over => game.reset(),
                _ => {}
            }
        }

        if let Some(update_args) = e.update_args() {
            game.update(update_args.dt);
        }

        window.draw_2d(&e, |c, g, _device| {
            clear([0.0, 0.0, 0.0, 1.0], g);

            // Draw snake
            for segment in &game.snake {
                let x = segment.x * CELL_SIZE;
                let y = segment.y * CELL_SIZE;
                rectangle(
                    [0.0, 1.0, 0.0, 1.0], // Green color
                    [x as f64, y as f64, CELL_SIZE as f64, CELL_SIZE as f64],
                    c.transform,
                    g,
                );
            }

            // Draw food
            let food_x = game.food.x * CELL_SIZE;
            let food_y = game.food.y * CELL_SIZE;
            rectangle(
                [1.0, 0.0, 0.0, 1.0], // Red color
                [food_x as f64, food_y as f64, CELL_SIZE as f64, CELL_SIZE as f64],
                c.transform,
                g,
            );

            // Simple version without text rendering
            // Just draw the score as blocks
            for i in 0..game.score {
                rectangle(
                    [1.0, 1.0, 0.0, 1.0], // Yellow color
                    [10.0 + (i as f64 * 15.0), 10.0, 10.0, 10.0],
                    c.transform,
                    g,
                );
            }

            // Draw game over indicator
            if game.is_game_over {
                rectangle(
                    [1.0, 0.0, 0.0, 0.5], // Red with transparency
                    [0.0, 0.0, WIDTH as f64, HEIGHT as f64],
                    c.transform,
                    g,
                );
            }
        });
    }
}