use crate::map::{CellType, Map, RobotExplorationUpdate}; // Updated import

pub const INITIAL_ROBOT_ENERGY: u32 = 100;

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
    pub pending_exploration_updates: RobotExplorationUpdate, // Added field
}

impl Robot {
    // Create a new robot at a given position
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            energy: INITIAL_ROBOT_ENERGY, // Use constant
            minerals: 0,
            science_points: 0,
            pending_exploration_updates: Vec::new(), // Initialize field
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
    pub fn explore(&mut self, map: &mut Map) -> bool { // Changed to &mut self
        let (current_x, current_y) = (self.x, self.y);
        // map.explore marks the cell as explored by the map system
        // and returns true if the exploration attempt was valid/changed state.
        if map.explore(current_x, current_y) {
            // If explored successfully, get the cell's data to add to robot's pending updates.
            if let Some(cell_data) = map.get_cell(current_x, current_y) {
                self.pending_exploration_updates.push(((current_x, current_y), cell_data.cell_type.clone()));
            }
            true
        } else {
            false
        }
    }

    // Method for the robot to unload its collected payload
    pub fn unload_payload(&mut self) -> (u32, u32, u32) {
        let energy_payload = self.energy.saturating_sub(INITIAL_ROBOT_ENERGY);
        // The robot keeps at least INITIAL_ROBOT_ENERGY if it had more,
        // or its current energy if it was already below INITIAL_ROBOT_ENERGY.
        self.energy = self.energy.saturating_sub(energy_payload);

        let minerals_payload = self.minerals;
        self.minerals = 0;

        let science_payload = self.science_points;
        self.science_points = 0;

        (energy_payload, minerals_payload, science_payload)
    }

    // Method for the robot to provide its exploration updates
    pub fn get_exploration_updates(&mut self) -> RobotExplorationUpdate {
        std::mem::take(&mut self.pending_exploration_updates)
    }

    // Check if the robot has still energy
    pub fn is_active(&self) -> bool {
        self.energy > 0
    }

    // Display the robot's statistics
    pub fn display_stats(&self) -> String {
        format!(
            "Robot at ({}, {}) | Energy: {} | Minerals: {} | Science Points: {} | Updates: {}",
            self.x, self.y, self.energy, self.minerals, self.science_points, self.pending_exploration_updates.len()
        )
    }
}
