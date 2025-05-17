use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::io::{stdout, Result, Write};

// Types of cells on the map
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellType {
    Empty,
    Obstacle,
    Energy(u32),
    Mineral(u32),
    SciencePoint,
}

// Structure representing a cell of the map
#[derive(Debug, Clone)]
pub struct Cell {
    pub cell_type: CellType,
    pub explored: bool,
}

impl Cell {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            cell_type,
            explored: false,
        }
    }
}

// Main structure of the map
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
    pub seed: u32,
}

impl Map {
    // Create a new map with specified width, height, and seed
    pub fn new(width: usize, height: usize, seed: u32) -> Self {
        let mut map = Self {
            width,
            height,
            cells: vec![vec![Cell::new(CellType::Empty); width]; height],
            seed,
        };
        map.generate();
        map
    }

    // Generate the map with obstacles and resources
    fn generate(&mut self) {
        let perlin = Perlin::new(self.seed);
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed as u64);

        // Generation of obstacles with Perlin noise
        for y in 0..self.height {
            for x in 0..self.width {
                let nx = x as f64 / self.width as f64 * 5.0;
                let ny = y as f64 / self.height as f64 * 5.0;
                let noise_val = perlin.get([nx, ny]);

                // High noise values become obstacles
                if noise_val > 0.3 {
                    self.cells[y][x].cell_type = CellType::Obstacle;
                }
            }
        }

        // Placement of energy resources
        self.place_resources(&mut rng, self.width * self.height / 20, |amount| {
            CellType::Energy(amount)
        });

        // Placement of mineral resources
        self.place_resources(&mut rng, self.width * self.height / 30, |amount| {
            CellType::Mineral(amount)
        });

        // Placement of scientific interest points
        self.place_resources(&mut rng, self.width * self.height / 50, |_| {
            CellType::SciencePoint
        });
    }

    // Place random resources on the map
    fn place_resources<F>(&mut self, rng: &mut ChaCha8Rng, count: usize, resource_creator: F)
    where
        F: Fn(u32) -> CellType,
    {
        let mut placed = 0;
        while placed < count {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            if let CellType::Empty = self.cells[y][x].cell_type {
                let amount = rng.gen_range(10..100);
                self.cells[y][x].cell_type = resource_creator(amount);
                placed += 1;
            }
        }
    }

    // Display the map in the terminal
    pub fn display(&self) -> Result<()> {
        let mut stdout = stdout();

        // Do not clear the screen at each frame
        // execute!(stdout, Clear(ClearType::All))?;

        // Define colors for each cell type
        let colors = [
            (&CellType::Empty, (Color::Black, Color::Black, " ")),
            (&CellType::Obstacle, (Color::Grey, Color::DarkGrey, "▓")),
            (&CellType::Energy(0), (Color::Yellow, Color::Black, "E")),
            (&CellType::Mineral(0), (Color::Blue, Color::Black, "M")),
            (&CellType::SciencePoint, (Color::Green, Color::Black, "S")),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();

        // Display each cell without clearing the screen
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y][x];
                let cell_type = match &cell.cell_type {
                    CellType::Energy(_) => &CellType::Energy(0),
                    CellType::Mineral(_) => &CellType::Mineral(0),
                    other => other,
                };

                let (fg, bg, symbol) =
                    colors
                        .get(cell_type)
                        .unwrap_or(&(Color::White, Color::Black, "?"));

                execute!(
                    stdout,
                    MoveTo(x as u16, y as u16),
                    SetForegroundColor(*fg),
                    SetBackgroundColor(*bg),
                )?;

                write!(stdout, "{}", symbol)?;
            }
        }

        // Reset colors
        execute!(stdout, ResetColor)?;

        // Flush once at the end
        stdout.flush()?;
        Ok(())
    }

    // Check if the coordinates are valid
    pub fn is_valid_position(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    // Get a reference to a cell
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if self.is_valid_position(x, y) {
            Some(&self.cells[y][x])
        } else {
            None
        }
    }

    // Get a mutable reference to a cell
    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        if self.is_valid_position(x, y) {
            Some(&mut self.cells[y][x])
        } else {
            None
        }
    }

    // Mark a cell as explored
    pub fn explore(&mut self, x: usize, y: usize) -> bool {
        if let Some(cell) = self.get_cell_mut(x, y) {
            cell.explored = true;
            true
        } else {
            false
        }
    }

    // Try to collect resources at a given position
    pub fn collect_resource(&mut self, x: usize, y: usize) -> Option<(CellType, u32)> {
        if let Some(cell) = self.get_cell_mut(x, y) {
            match cell.cell_type {
                CellType::Energy(amount) => {
                    cell.cell_type = CellType::Empty;
                    Some((CellType::Energy(0), amount))
                }
                CellType::Mineral(amount) => {
                    cell.cell_type = CellType::Empty;
                    Some((CellType::Mineral(0), amount))
                }
                CellType::SciencePoint => {
                    cell.cell_type = CellType::Empty;
                    Some((CellType::SciencePoint, 1))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
