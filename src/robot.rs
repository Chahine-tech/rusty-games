use crate::map::{CellType, Map};

// Direction de déplacement du robot
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

// Structure representing an exploration robot
pub struct Robot {
    pub x: usize,
    pub y: usize,
    pub energy: u32,
    pub minerals: u32,
    pub science_points: u32,
}

impl Robot {
    // Create a new robot at a given position
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            energy: 100,
            minerals: 0,
            science_points: 0,
        }
    }

    // Move the robot in a given direction
    pub fn move_in_direction(&mut self, direction: Direction, map: &Map) -> bool {
        let (new_x, new_y) = match direction {
            Direction::North => {
                if self.y == 0 {
                    return false;
                }
                (self.x, self.y - 1)
            }
            Direction::East => {
                if self.x >= map.width - 1 {
                    return false;
                }
                (self.x + 1, self.y)
            }
            Direction::South => {
                if self.y >= map.height - 1 {
                    return false;
                }
                (self.x, self.y + 1)
            }
            Direction::West => {
                if self.x == 0 {
                    return false;
                }
                (self.x - 1, self.y)
            }
        };

        // Check if the new position is valid and is not an obstacle
        if let Some(cell) = map.get_cell(new_x, new_y) {
            if let CellType::Obstacle = cell.cell_type {
                return false;
            }

            // Move the robot and consume energy
            self.x = new_x;
            self.y = new_y;
            self.energy = self.energy.saturating_sub(1);
            true
        } else {
            false
        }
    }

    // Collect resources at the current position
    pub fn collect_resource(&mut self, map: &mut Map) -> bool {
        if let Some((resource_type, amount)) = map.collect_resource(self.x, self.y) {
            match resource_type {
                CellType::Energy(_) => {
                    self.energy += amount;
                }
                CellType::Mineral(_) => {
                    self.minerals += amount;
                }
                CellType::SciencePoint => {
                    self.science_points += amount;
                }
                _ => return false,
            }
            true
        } else {
            false
        }
    }

    // Explore the current cell
    pub fn explore(&self, map: &mut Map) -> bool {
        map.explore(self.x, self.y)
    }

    // Check if the robot has still energy
    pub fn is_active(&self) -> bool {
        self.energy > 0
    }

    // Display the robot's statistics
    pub fn display_stats(&self) -> String {
        format!(
            "Robot at ({}, {}) | Energy: {} | Minerals: {} | Science Points: {}",
            self.x, self.y, self.energy, self.minerals, self.science_points
        )
    }
}
