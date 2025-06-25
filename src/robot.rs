use crate::map::{CellType, Map, RobotExplorationUpdate}; // Updated import
use rand::Rng;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

pub const INITIAL_ROBOT_ENERGY: u32 = 100;

// A* pathfinding node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PathNode {
    x: usize,
    y: usize,
    g_cost: u32,  // Cost from start
    h_cost: u32,  // Heuristic cost to goal
    f_cost: u32,  // Total cost (g + h)
}

impl PathNode {
    fn new(x: usize, y: usize, g_cost: u32, h_cost: u32) -> Self {
        Self {
            x,
            y,
            g_cost,
            h_cost,
            f_cost: g_cost + h_cost,
        }
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.f_cost.cmp(&self.f_cost)
            .then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Direction de déplacement du robot
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

// Different types of robots with specialized behaviors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RobotType {
    Explorer,        // Focuses on exploring unknown areas
    EnergyCollector, // Prioritizes energy collection
    MineralCollector, // Prioritizes mineral collection
    Scientist,       // Focuses on science points
}

// Robot behavior state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RobotState {
    Exploring,
    ReturningToStation,
    AtStation,
}

// Structure representing an exploration robot
#[derive(Clone)]
pub struct Robot {
    pub x: usize,
    pub y: usize,
    pub energy: u32,
    pub minerals: u32,
    pub science_points: u32,
    pub pending_exploration_updates: RobotExplorationUpdate, // Added field
    pub robot_type: RobotType, // New field
    pub state: RobotState,     // New field
    pub target_x: Option<usize>, // Target coordinates for pathfinding
    pub target_y: Option<usize>,
    pub steps_since_last_find: u32, // For exploration strategy
}

impl Robot {
    // Create a new robot at a given position with a specific type
    #[allow(dead_code)]
    pub fn new(x: usize, y: usize) -> Self {
        Self::new_with_type(x, y, RobotType::Explorer)
    }

    // Create a new robot with a specific type
    pub fn new_with_type(x: usize, y: usize, robot_type: RobotType) -> Self {
        Self {
            x,
            y,
            energy: INITIAL_ROBOT_ENERGY,
            minerals: 0,
            science_points: 0,
            pending_exploration_updates: Vec::new(),
            robot_type,
            state: RobotState::Exploring,
            target_x: None,
            target_y: None,
            steps_since_last_find: 0,
        }
    }

    // Autonomous behavior - main AI loop
    pub fn autonomous_update(&mut self, map: &mut Map, station_x: usize, station_y: usize, other_robots: &[Robot]) {
        // Skip update if robot has no energy
        if self.energy == 0 {
            return;
        }

        match self.state {
            RobotState::Exploring => {
                self.autonomous_explore(map, station_x, station_y, other_robots);
            }
            RobotState::ReturningToStation => {
                self.move_towards_station(map, station_x, station_y, other_robots);
            }
            RobotState::AtStation => {
                // Robot is at station, will be handled by main loop
                // Reset state to exploring after interaction
                self.state = RobotState::Exploring;
            }
        }
    }

    // Check if robot should return to station
    fn should_return_to_station(&self) -> bool {
        // Return if energy is critically low
        if self.energy <= 20 { // Slightly increased for safety
            return true;
        }

        // Return based on robot type and cargo
        match self.robot_type {
            RobotType::Explorer => {
                // Let explorers venture further but return before energy gets too low
                self.energy <= 25 || self.pending_exploration_updates.len() > 30 // Balanced thresholds
            }
            RobotType::EnergyCollector => {
                // Return when carrying significant energy or low on energy
                self.energy <= 25 || (self.energy > INITIAL_ROBOT_ENERGY + 70) // Slightly reduced cargo threshold
            }
            RobotType::MineralCollector => {
                // Return when carrying minerals or energy is low
                self.minerals > 35 || self.energy <= 25 // Slightly reduced thresholds
            }
            RobotType::Scientist => {
                // Return when has science points or energy is low
                self.science_points > 6 || self.energy <= 25 // Slightly reduced thresholds
            }
        }
    }

    // Autonomous exploration based on robot type
    fn autonomous_explore(&mut self, map: &mut Map, station_x: usize, station_y: usize, other_robots: &[Robot]) {
        // Check if robot should return to station
        if self.should_return_to_station() {
            self.state = RobotState::ReturningToStation;
            self.target_x = Some(station_x);
            self.target_y = Some(station_y);
            return;
        }

        // Try to collect resource at current position first
        self.collect_resource(map);
        
        // Explore current position
        self.explore(map);

        // Choose next move based on robot type
        let next_direction = match self.robot_type {
            RobotType::Explorer => self.choose_explorer_direction(map, other_robots),
            RobotType::EnergyCollector => self.choose_energy_collector_direction(map, other_robots),
            RobotType::MineralCollector => self.choose_mineral_collector_direction(map, other_robots),
            RobotType::Scientist => self.choose_scientist_direction(map, other_robots),
        };

        if let Some(direction) = next_direction {
            if self.move_in_direction(direction, map, other_robots) {
                if self.found_something_at_current_position(map) {
                    self.steps_since_last_find = 0;
                } else {
                    self.steps_since_last_find += 1;
                }
            } else {
                self.steps_since_last_find += 1;
            }
        } else {
            // No good direction found, try random movement
            if !self.move_randomly(map, other_robots) {
                // Even random movement failed, increment stuck counter
                self.steps_since_last_find += 1;
            }
        }

        // If stuck for too long, try teleporting to a nearby free space
        if self.steps_since_last_find > 5 { // Reduced from 8 to 5 for more aggressive unstuck
            self.try_unstuck(map, other_robots);
        }
    }

    // Try to get unstuck by finding a nearby free position
    fn try_unstuck(&mut self, map: &Map, other_robots: &[Robot]) {
        // Try to find a completely unexplored area to jump to
        let mut best_position = None;
        let mut best_score = -1i32;
        
        // Search in a much wider radius for unexplored areas  
        for radius in 8..=25 { // Increased search radius significantly
            for angle in 0..16 { // More angles for better coverage
                let angle_rad = angle as f32 * std::f32::consts::PI / 8.0;
                let dx = (radius as f32 * angle_rad.cos()) as i32;
                let dy = (radius as f32 * angle_rad.sin()) as i32;
                
                let new_x = (self.x as i32 + dx).max(0).min(map.width as i32 - 1) as usize;
                let new_y = (self.y as i32 + dy).max(0).min(map.height as i32 - 1) as usize;
                
                if let Some(cell) = map.get_cell(new_x, new_y) {
                    if cell.cell_type != CellType::Obstacle && 
                       !other_robots.iter().any(|r| r.x == new_x && r.y == new_y && r.energy > 0) {
                        
                        let mut score = 0;
                        if !cell.explored {
                            score += 150; // Higher reward for unexplored
                        }
                        
                        // Count unexplored neighbors in a wider area
                        for dy_check in -2..=2 {
                            for dx_check in -2..=2 {
                                let check_x = (new_x as i32 + dx_check).max(0).min(map.width as i32 - 1) as usize;
                                let check_y = (new_y as i32 + dy_check).max(0).min(map.height as i32 - 1) as usize;
                                
                                if let Some(neighbor) = map.get_cell(check_x, check_y) {
                                    if !neighbor.explored {
                                        score += 15; // Bonus for unexplored neighbors
                                    }
                                }
                            }
                        }
                        
                        // Bonus for being far from current position (encourage long jumps)
                        let distance_from_current = ((new_x as i32 - self.x as i32).abs() + (new_y as i32 - self.y as i32).abs()) as i32;
                        score += distance_from_current;
                        
                        if score > best_score {
                            best_score = score;
                            best_position = Some((new_x, new_y));
                        }
                    }
                }
            }
            
            // If we found a great position, break early
            if best_position.is_some() && best_score > 200 {
                break;
            }
        }
        
        // Teleport to the best position found
        if let Some((new_x, new_y)) = best_position {
            self.x = new_x;
            self.y = new_y;
            self.steps_since_last_find = 0;
            // Small energy cost for teleportation
            self.energy = self.energy.saturating_sub(3);
        }
    }

    // Check if current position has something of interest
    fn found_something_at_current_position(&self, map: &Map) -> bool {
        if let Some(cell) = map.get_cell(self.x, self.y) {
            match cell.cell_type {
                CellType::Energy(_) | CellType::Mineral(_) | CellType::SciencePoint => true,
                _ => false,
            }
        } else {
            false
        }
    }

    // Explorer: prioritizes unexplored areas
    fn choose_explorer_direction(&self, map: &Map, other_robots: &[Robot]) -> Option<Direction> {
        let directions = [Direction::North, Direction::East, Direction::South, Direction::West];
        let mut best_direction = None;
        let mut best_score = -1000i32; // Lower threshold to encourage more movement

        for direction in directions {
            if let Some((new_x, new_y)) = self.get_next_position(direction, map) {
                if self.is_valid_move(new_x, new_y, map, other_robots) {
                    let score = self.calculate_explorer_score(new_x, new_y, map);
                    if score > best_score {
                        best_score = score;
                        best_direction = Some(direction);
                    }
                }
            }
        }
        
        // If no good direction found, encourage movement away from station
        if best_direction.is_none() || best_score < 0 {
            return self.choose_direction_away_from_explored_areas(map, other_robots);
        }
        
        best_direction
    }

    // Energy collector: prioritizes energy sources
    fn choose_energy_collector_direction(&self, map: &Map, other_robots: &[Robot]) -> Option<Direction> {
        self.choose_resource_direction(map, other_robots, |cell_type| {
            matches!(cell_type, CellType::Energy(_))
        })
    }

    // Mineral collector: prioritizes mineral sources
    fn choose_mineral_collector_direction(&self, map: &Map, other_robots: &[Robot]) -> Option<Direction> {
        self.choose_resource_direction(map, other_robots, |cell_type| {
            matches!(cell_type, CellType::Mineral(_))
        })
    }

    // Scientist: prioritizes science points
    fn choose_scientist_direction(&self, map: &Map, other_robots: &[Robot]) -> Option<Direction> {
        self.choose_resource_direction(map, other_robots, |cell_type| {
            matches!(cell_type, CellType::SciencePoint)
        })
    }

    // Generic resource-seeking behavior
    fn choose_resource_direction<F>(&self, map: &Map, other_robots: &[Robot], is_target: F) -> Option<Direction>
    where
        F: Fn(&CellType) -> bool,
    {
        let directions = [Direction::North, Direction::East, Direction::South, Direction::West];
        let mut best_direction = None;
        let mut best_score = -1i32;

        for direction in directions {
            if let Some((new_x, new_y)) = self.get_next_position(direction, map) {
                if self.is_valid_move(new_x, new_y, map, other_robots) {
                    let score = self.calculate_resource_score(new_x, new_y, map, &is_target);
                    if score > best_score {
                        best_score = score;
                        best_direction = Some(direction);
                    }
                }
            }
        }
        best_direction
    }

    // Calculate score for explorer (prioritizes unexplored areas)
    fn calculate_explorer_score(&self, x: usize, y: usize, map: &Map) -> i32 {
        let mut score = 0;
        
        if let Some(cell) = map.get_cell(x, y) {
            // Heavily reward unexplored cells
            if !cell.explored {
                score += 150; // Increased reward
            } else {
                score -= 30; // Increased penalty for explored areas
            }
            
            // Calculate distance from current position (not from start)
            let distance_from_current = ((x as i32 - self.x as i32).abs() + (y as i32 - self.y as i32).abs()) as i32;
            
            // Bonus for moving away from current position (encourage exploration)
            score += distance_from_current * 3;
            
            // Look around for unexplored neighbors in a wider area
            let mut unexplored_neighbors = 0;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    if dx == 0 && dy == 0 { continue; }
                    let check_x = (x as i32 + dx).max(0).min(map.width as i32 - 1) as usize;
                    let check_y = (y as i32 + dy).max(0).min(map.height as i32 - 1) as usize;
                    
                    if let Some(neighbor_cell) = map.get_cell(check_x, check_y) {
                        if !neighbor_cell.explored {
                            unexplored_neighbors += 1;
                        }
                    }
                }
            }
            
            // Reward positions with many unexplored neighbors
            score += unexplored_neighbors * 15;
            
            // Extra bonus for edge positions (likely to lead to new areas)
            if x == 0 || x == map.width - 1 || y == 0 || y == map.height - 1 {
                score += 40;
            }
            
            // Bonus for corner positions (often unexplored)
            if (x == 0 || x == map.width - 1) && (y == 0 || y == map.height - 1) {
                score += 20;
            }
        }
        
        score
    }

    // Calculate score for resource collectors
    fn calculate_resource_score<F>(&self, x: usize, y: usize, map: &Map, is_target: &F) -> i32
    where
        F: Fn(&CellType) -> bool,
    {
        let mut score = 0;

        // Check current cell
        if let Some(cell) = map.get_cell(x, y) {
            if is_target(&cell.cell_type) {
                score += 25; // High priority for target resource
            }
            
            if !cell.explored {
                score += 5; // Exploration value for resource collectors too
            }
        }

        // Check surrounding cells for target resources (wider radius)
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx == 0 && dy == 0 { continue; }
                let check_x = (x as i32 + dx) as usize;
                let check_y = (y as i32 + dy) as usize;
                
                if let Some(cell) = map.get_cell(check_x, check_y) {
                    if is_target(&cell.cell_type) {
                        let distance = (dx.abs() + dy.abs()) as i32;
                        score += 8 - distance; // Closer resources get higher score
                    }
                    if !cell.explored {
                        score += 1; // Small exploration bonus
                    }
                }
            }
        }

        // Add randomness to prevent clustering
        let mut rng = rand::thread_rng();
        score += rng.gen_range(-1..=1);

        score
    }

    // Move towards station using A* pathfinding
    fn move_towards_station(&mut self, map: &mut Map, station_x: usize, station_y: usize, other_robots: &[Robot]) {
        // Check if already at station
        if self.x == station_x && self.y == station_y {
            self.state = RobotState::AtStation;
            return;
        }

        // Use A* pathfinding to find optimal path
        if let Some(path) = self.find_path(self.x, self.y, station_x, station_y, map, other_robots) {
            // If path found and has more than one step (current position + next step)
            if path.len() > 1 {
                let next_pos = path[1]; // Skip current position (path[0])
                let direction = self.get_direction_to_position(next_pos.0, next_pos.1);
                
                if let Some(dir) = direction {
                    if self.move_in_direction(dir, map, other_robots) {
                        return;
                    }
                }
            }
        }
        
        // Fallback to simple directional movement if A* fails
        let dx = if self.x < station_x { 1 } else if self.x > station_x { -1 } else { 0 };
        let dy = if self.y < station_y { 1 } else if self.y > station_y { -1 } else { 0 };

        let directions = if dx > 0 && dy > 0 {
            vec![Direction::East, Direction::South, Direction::North, Direction::West]
        } else if dx > 0 && dy < 0 {
            vec![Direction::East, Direction::North, Direction::South, Direction::West]
        } else if dx < 0 && dy > 0 {
            vec![Direction::West, Direction::South, Direction::North, Direction::East]
        } else if dx < 0 && dy < 0 {
            vec![Direction::West, Direction::North, Direction::South, Direction::East]
        } else if dx > 0 {
            vec![Direction::East, Direction::North, Direction::South, Direction::West]
        } else if dx < 0 {
            vec![Direction::West, Direction::North, Direction::South, Direction::East]
        } else if dy > 0 {
            vec![Direction::South, Direction::East, Direction::West, Direction::North]
        } else {
            vec![Direction::North, Direction::East, Direction::West, Direction::South]
        };

        // Try directions in order of preference
        for direction in directions {
            if self.move_in_direction(direction, map, other_robots) {
                return;
            }
        }
        
        // If all directions failed, try random movement as last resort
        self.move_randomly(map, other_robots);
    }

    // Get next position for a given direction
    fn get_next_position(&self, direction: Direction, map: &Map) -> Option<(usize, usize)> {
        match direction {
            Direction::North => {
                if self.y == 0 { None } else { Some((self.x, self.y - 1)) }
            }
            Direction::East => {
                if self.x >= map.width - 1 { None } else { Some((self.x + 1, self.y)) }
            }
            Direction::South => {
                if self.y >= map.height - 1 { None } else { Some((self.x, self.y + 1)) }
            }
            Direction::West => {
                if self.x == 0 { None } else { Some((self.x - 1, self.y)) }
            }
        }
    }

    // Check if a move to given coordinates is valid
    fn is_valid_move(&self, x: usize, y: usize, map: &Map, other_robots: &[Robot]) -> bool {
        if let Some(cell) = map.get_cell(x, y) {
            // Check for obstacles
            if matches!(cell.cell_type, CellType::Obstacle) {
                return false;
            }
            
            // Check for other robots at the same position
            for robot in other_robots {
                if robot.x == x && robot.y == y && robot.energy > 0 {
                    return false;
                }
            }
            
            true
        } else {
            false
        }
    }

    // Move randomly when no better option is available
    fn move_randomly(&mut self, map: &mut Map, other_robots: &[Robot]) -> bool {
        let directions = [Direction::North, Direction::East, Direction::South, Direction::West];
        let mut rng = rand::thread_rng();
        
        // Shuffle directions and try them
        let mut shuffled_directions = directions;
        for i in 0..shuffled_directions.len() {
            let j = rng.gen_range(0..shuffled_directions.len());
            shuffled_directions.swap(i, j);
        }

        for direction in shuffled_directions {
            if self.move_in_direction(direction, map, other_robots) {
                return true;
            }
        }
        false
    }

    // Move the robot in a given direction
    pub fn move_in_direction(&mut self, direction: Direction, map: &Map, other_robots: &[Robot]) -> bool {
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

        // Check if the new position is valid
        if self.is_valid_move(new_x, new_y, map, other_robots) {
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
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.energy > 0
    }

    // Display the robot's statistics
    #[allow(dead_code)]
    pub fn display_stats(&self) -> String {
        format!(
            "Robot at ({}, {}) | Energy: {} | Minerals: {} | Science Points: {} | Updates: {}",
            self.x, self.y, self.energy, self.minerals, self.science_points, self.pending_exploration_updates.len()
        )
    }

    // Add this new method to encourage exploration away from known areas
    fn choose_direction_away_from_explored_areas(&self, map: &Map, other_robots: &[Robot]) -> Option<Direction> {
        let directions = [Direction::North, Direction::East, Direction::South, Direction::West];
        let mut best_direction = None;
        let mut max_unexplored_potential = 0;

        for direction in directions {
            if let Some((new_x, new_y)) = self.get_next_position(direction, map) {
                if self.is_valid_move(new_x, new_y, map, other_robots) {
                    // Calculate potential for finding unexplored areas in this direction
                    let potential = self.calculate_unexplored_potential(new_x, new_y, direction, map);
                    if potential > max_unexplored_potential {
                        max_unexplored_potential = potential;
                        best_direction = Some(direction);
                    }
                }
            }
        }
        
        best_direction
    }

    fn calculate_unexplored_potential(&self, x: usize, y: usize, direction: Direction, map: &Map) -> usize {
        let mut potential = 0;
        
        // Look ahead in this direction for unexplored clusters
        let mut current_x = x;
        let mut current_y = y;
        
        for step in 1..=15 { // Look further ahead
            if let Some((next_x, next_y)) = self.get_next_position_from(current_x, current_y, direction, map) {
                current_x = next_x;
                current_y = next_y;
                
                if let Some(cell) = map.get_cell(current_x, current_y) {
                    if !cell.explored {
                        // Found an unexplored cell, count surrounding unexplored area
                        let cluster_size = self.count_unexplored_cluster(current_x, current_y, map);
                        potential += cluster_size * (16 - step); // Weight by inverse distance
                        break; // Found a good target
                    }
                }
            } else {
                // Hit a boundary - might be unexplored area beyond
                potential += 50;
                break;
            }
        }
        
        potential
    }

    fn count_unexplored_cluster(&self, center_x: usize, center_y: usize, map: &Map) -> usize {
        let mut count = 0;
        
        // Check a 5x5 area around the center
        for dy in -2..=2 {
            for dx in -2..=2 {
                let check_x = (center_x as i32 + dx).max(0).min(map.width as i32 - 1) as usize;
                let check_y = (center_y as i32 + dy).max(0).min(map.height as i32 - 1) as usize;
                
                if let Some(cell) = map.get_cell(check_x, check_y) {
                    if !cell.explored && cell.cell_type != CellType::Obstacle {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }

    fn get_next_position_from(&self, x: usize, y: usize, direction: Direction, map: &Map) -> Option<(usize, usize)> {
        match direction {
            Direction::North => {
                if y == 0 { None } else { Some((x, y - 1)) }
            }
            Direction::East => {
                if x >= map.width - 1 { None } else { Some((x + 1, y)) }
            }
            Direction::South => {
                if y >= map.height - 1 { None } else { Some((x, y + 1)) }
            }
            Direction::West => {
                if x == 0 { None } else { Some((x - 1, y)) }
            }
        }
    }

    // Get direction from current position to target position
    fn get_direction_to_position(&self, target_x: usize, target_y: usize) -> Option<Direction> {
        let dx = target_x as i32 - self.x as i32;
        let dy = target_y as i32 - self.y as i32;
        
        match (dx.signum(), dy.signum()) {
            (1, 0) => Some(Direction::East),
            (-1, 0) => Some(Direction::West),
            (0, 1) => Some(Direction::South),
            (0, -1) => Some(Direction::North),
            _ => None, // Diagonal or same position
        }
    }

    // A* pathfinding implementation
    fn find_path(&self, start_x: usize, start_y: usize, goal_x: usize, goal_y: usize, map: &Map, other_robots: &[Robot]) -> Option<Vec<(usize, usize)>> {
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        
        let start_node = PathNode::new(start_x, start_y, 0, self.heuristic(start_x, start_y, goal_x, goal_y));
        open_set.push(start_node);
        g_score.insert((start_x, start_y), 0);
        
        while let Some(current) = open_set.pop() {
            // If we reached the goal
            if current.x == goal_x && current.y == goal_y {
                return Some(self.reconstruct_path(came_from, (current.x, current.y)));
            }
            
            // Check all neighbors
            let neighbors = [
                (current.x.wrapping_sub(1), current.y), // West
                (current.x + 1, current.y),             // East
                (current.x, current.y.wrapping_sub(1)), // North
                (current.x, current.y + 1),             // South
            ];
            
            for (nx, ny) in neighbors {
                // Skip invalid positions
                if nx >= map.width || ny >= map.height {
                    continue;
                }
                
                // Skip obstacles and other robots
                if !self.is_valid_move(nx, ny, map, other_robots) {
                    continue;
                }
                
                let tentative_g_score = g_score.get(&(current.x, current.y)).unwrap_or(&u32::MAX) + 1;
                let current_g_score = g_score.get(&(nx, ny)).unwrap_or(&u32::MAX);
                
                if tentative_g_score < *current_g_score {
                    came_from.insert((nx, ny), (current.x, current.y));
                    g_score.insert((nx, ny), tentative_g_score);
                    
                    let h_cost = self.heuristic(nx, ny, goal_x, goal_y);
                    let neighbor_node = PathNode::new(nx, ny, tentative_g_score, h_cost);
                    open_set.push(neighbor_node);
                }
            }
        }
        
        None // No path found
    }
    
    // Manhattan distance heuristic
    fn heuristic(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> u32 {
        ((x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()) as u32
    }
    
    // Reconstruct path from came_from map
    fn reconstruct_path(&self, came_from: HashMap<(usize, usize), (usize, usize)>, mut current: (usize, usize)) -> Vec<(usize, usize)> {
        let mut path = vec![current];
        
        while let Some(&parent) = came_from.get(&current) {
            current = parent;
            path.push(current);
        }
        
        path.reverse();
        path
    }
}
