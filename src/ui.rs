use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Result, Write};
use std::time::Duration;

use crate::map::Map;
use crate::robot::{Direction, Robot};

// Structure to manage the user interface
pub struct UI {
    pub width: usize,
    pub height: usize,
    last_robot_x: usize,
    last_robot_y: usize,
}

impl UI {
    // Create a new user interface
    pub fn new() -> Result<Self> {
        // Get the terminal size
        let (width, height) = terminal::size()?;

        // Configure the terminal
        terminal::enable_raw_mode()?;

        let mut stdout = stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All) // Clear the screen only at startup
        )?;

        Ok(Self {
            width: width as usize,
            height: height as usize,
            last_robot_x: 0,
            last_robot_y: 0,
        })
    }

    // Clean up and restore the terminal
    pub fn cleanup(&self) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen, Show)?;
        Ok(())
    }

    // Display the map and the robot's information
    pub fn render(&mut self, map: &Map, robot: &Robot) -> Result<()> {
        let mut stdout = stdout();

        // Display the map only at the first render or if necessary
        static mut FIRST_RENDER: bool = true;
        unsafe {
            if FIRST_RENDER {
                map.display()?;
                FIRST_RENDER = false;
            }
        }

        // Clear the robot's old position if it has changed
        if self.last_robot_x != robot.x || self.last_robot_y != robot.y {
            // Restore the cell at the old position
            if let Some(cell) = map.get_cell(self.last_robot_x, self.last_robot_y) {
                let symbol = match cell.cell_type {
                    crate::map::CellType::Empty => " ",
                    crate::map::CellType::Obstacle => "▓",
                    crate::map::CellType::Energy(_) => "E",
                    crate::map::CellType::Mineral(_) => "M",
                    crate::map::CellType::SciencePoint => "S",
                };

                let color = match cell.cell_type {
                    crate::map::CellType::Empty => Color::Black,
                    crate::map::CellType::Obstacle => Color::Grey,
                    crate::map::CellType::Energy(_) => Color::Yellow,
                    crate::map::CellType::Mineral(_) => Color::Blue,
                    crate::map::CellType::SciencePoint => Color::Green,
                };

                execute!(
                    stdout,
                    MoveTo(self.last_robot_x as u16, self.last_robot_y as u16),
                    SetForegroundColor(color),
                    SetBackgroundColor(Color::Black),
                )?;
                write!(stdout, "{}", symbol)?;
            }

            // Update the last known position
            self.last_robot_x = robot.x;
            self.last_robot_y = robot.y;
        }

        // Display the robot at its current position
        execute!(
            stdout,
            MoveTo(robot.x as u16, robot.y as u16),
            SetForegroundColor(Color::Red),
        )?;
        write!(stdout, "R")?;

        // Display the robot's statistics at the bottom of the screen
        execute!(
            stdout,
            MoveTo(0, self.height as u16 - 1),
            SetForegroundColor(Color::White),
        )?;
        write!(stdout, "{}", robot.display_stats())?;

        // Display the instructions
        execute!(
            stdout,
            MoveTo(0, self.height as u16 - 2),
            SetForegroundColor(Color::Cyan),
        )?;
        write!(
            stdout,
            "Controls: Arrows to move, E to explore, C to collect, Q to quit"
        )?;

        execute!(stdout, ResetColor)?;
        stdout.flush()?;

        Ok(())
    }

    // Wait and process a user input
    pub fn handle_input(&self, robot: &mut Robot, map: &mut Map) -> Result<bool> {
        // Increase the waiting time to reduce polling frequency
        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        robot.move_in_direction(Direction::North, map);
                    }
                    KeyCode::Right => {
                        robot.move_in_direction(Direction::East, map);
                    }
                    KeyCode::Down => {
                        robot.move_in_direction(Direction::South, map);
                    }
                    KeyCode::Left => {
                        robot.move_in_direction(Direction::West, map);
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        robot.explore(map);
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        robot.collect_resource(map);
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }

        Ok(true)
    }
}
