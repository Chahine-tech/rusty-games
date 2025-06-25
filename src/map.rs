use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// Types of cells on the map
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellType {
    Empty,
    Obstacle,
    Energy(u32),
    Mineral(u32),
    SciencePoint,
}

// Data structure for updates from robots
// Each entry is ((x, y_coordinates), type_of_cell)
pub type RobotExplorationUpdate = Vec<((usize, usize), CellType)>;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_creation() {
        let map = Map::new(10, 10, 123);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.seed, 123);
    }

    #[test]
    fn test_valid_position() {
        let map = Map::new(5, 5, 123);
        assert!(map.is_valid_position(0, 0));
        assert!(map.is_valid_position(4, 4));
        assert!(!map.is_valid_position(5, 5));
        assert!(!map.is_valid_position(10, 10));
    }

    #[test]
    fn test_get_cell() {
        let map = Map::new(3, 3, 123);
        assert!(map.get_cell(0, 0).is_some());
        assert!(map.get_cell(2, 2).is_some());
        assert!(map.get_cell(3, 3).is_none());
    }

    #[test]
    fn test_explore_cell() {
        let mut map = Map::new(3, 3, 123);
        assert!(map.explore(1, 1));
        assert!(!map.explore(5, 5)); // Invalid position

        if let Some(cell) = map.get_cell(1, 1) {
            assert!(cell.explored);
        }
    }

    #[test]
    fn test_collect_resource() {
        let mut map = Map::new(3, 3, 123);
        // Manually set a cell to have energy
        if let Some(cell) = map.get_cell_mut(1, 1) {
            cell.cell_type = CellType::Energy(50);
        }

        let result = map.collect_resource(1, 1);
        assert!(result.is_some());
        if let Some((cell_type, amount)) = result {
            assert_eq!(amount, 50);
            match cell_type {
                CellType::Energy(_) => {} // Expected
                _ => panic!("Expected Energy type"),
            }
        }

        // Cell should now be empty
        if let Some(cell) = map.get_cell(1, 1) {
            assert_eq!(cell.cell_type, CellType::Empty);
        }
    }
}
