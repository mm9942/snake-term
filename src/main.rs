extern crate crossterm;
extern crate tui;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction as TuiDirection, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: usize,
    y: usize,
}

struct Snake {
    body: VecDeque<Point>,
    direction: Direction,
}

struct App {
    snake: Snake,
    food: Vec<Point>,
    max_food: usize,
    bounds: Rect,
    last_update: Instant,
    should_exit: bool,
}

impl App {
    fn new(bounds: Rect) -> App {
        let mut snake_body = VecDeque::new();
        snake_body.push_back(Point {
            x: (bounds.width / 2) as usize,
            y: (bounds.height / 2) as usize,
        });

        let food = vec![Point {
            x: (bounds.width / 3) as usize,
            y: (bounds.height / 3) as usize,
        }];

        App {
            snake: Snake {
                body: snake_body,
                direction: Direction::Right,
            },
            food,
            max_food: 15,
            bounds,
            last_update: Instant::now(),
            should_exit: false,
        }
    }

    fn on_tick(&mut self) {
        let head = match self.snake.direction {
            Direction::Up => Point {
                x: self.snake.body.front().unwrap().x,
                y: self.snake.body.front().unwrap().y - 1,
            },
            Direction::Down => Point {
                x: self.snake.body.front().unwrap().x,
                y: self.snake.body.front().unwrap().y + 1,
            },
            Direction::Left => Point {
                x: self.snake.body.front().unwrap().x - 1,
                y: self.snake.body.front().unwrap().y,
            },
            Direction::Right => Point {
                x: self.snake.body.front().unwrap().x + 1,
                y: self.snake.body.front().unwrap().y,
            },
        };

        if self.snake.body.contains(&head)
            || head.x == 0
            || head.y == 0
            || head.x == self.bounds.width as usize - 1
            || head.y == self.bounds.height as usize - 1
        {
            self.should_exit = true;
            return;
        }

        let mut ate_food = false;
        for i in 0..self.food.len() {
            if head == self.food[i] {
                ate_food = true;
                self.food.remove(i);
                self.spawn_food(5);
                break;
            }
        }

        if !ate_food {
            self.snake.body.pop_back();
        }

        self.snake.body.push_front(head);
    }

    fn spawn_food(&mut self, count: usize) {
        let remaining_space = self.max_food.saturating_sub(self.food.len());
        let items_to_spawn = count.min(remaining_space);

        for _ in 0..items_to_spawn {
            loop {
                let new_food = Point {
                    x: (rand::random::<f32>() * self.bounds.width as f32) as usize,
                    y: (rand::random::<f32>() * self.bounds.height as f32) as usize,
                };
                if !self.snake.body.contains(&new_food) && !self.food.contains(&new_food) {
                    self.food.push(new_food);
                    break;
                }
            }
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        self.snake.direction = match key {
            KeyCode::Up => Direction::Up,
            KeyCode::Down => Direction::Down,
            KeyCode::Left => Direction::Left,
            KeyCode::Right => Direction::Right,
            KeyCode::Char('q') => {
                self.should_exit = true;
                self.snake.direction
            }
            _ => self.snake.direction,
        };
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let bounds = terminal.size()?;
    let mut app = App::new(bounds);

    Ok(loop {
        if app.should_exit {
            break;
        }

        terminal.draw(|f| {
            let snake = &app.snake;
            let food = &app.food;
            let rect = Rect::new(0, 0, bounds.width, bounds.height);

            let mut grid_string = String::new();

            for y in 0..bounds.height as usize {
                for x in 0..bounds.width as usize {
                    let point = Point { x, y };
                    if snake.body.contains(&point) {
                        grid_string.push('@');
                    } else if food.iter().any(|&food_point| food_point == point) {
                        grid_string.push('F');
                    } else {
                        grid_string.push('.');
                    }
                }
                grid_string.push('\n');
            }

            let paragraph =
                Paragraph::new(grid_string).block(Block::default().borders(Borders::ALL));
            f.render_widget(paragraph, rect);
        })?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        if Instant::now().duration_since(app.last_update) >= Duration::from_millis(100) {
            app.on_tick();
            app.last_update = Instant::now();
        }
    })
}
