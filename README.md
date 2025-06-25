# Rusty Swarm 🤖

**An Autonomous Robot Exploration Simulation in Rust**

Rusty Swarm is a terminal-based simulation game where you manage an autonomous swarm of specialized robots that explore, collect resources, and expand their knowledge of a procedurally generated world. Watch as your robots work together to map unknown territories, gather valuable resources, and grow their colony.

## 🌟 Features

- **🔍 Intelligent Swarm Exploration**: Robots autonomously explore the world using A* pathfinding and directional scoring strategies
- **⚡ Resource Management**: Collect energy, minerals, and science points to fuel your growing robot colony
- **🏭 Autonomous Robot Creation**: The station automatically creates new robots when resources allow
- **🗺️ Real-time Map Discovery**: Watch the world unfold as robots explore and share their findings
- **🎯 Specialized Robot Types**: Different robot classes with unique behaviors and priorities
- **📊 Live Statistics**: Real-time monitoring of your colony's progress and resources

## 🎮 Game Concept

You control a **Station** that manages a fleet of autonomous robots. Each robot operates independently but shares information with the colony. The goal is to explore as much of the world as possible while efficiently collecting resources to sustain and expand your robot fleet.

### Robot Types

1. **🔍 Explorer** - Prioritizes discovering unexplored areas and mapping new territories
2. **⚡ Energy Collector** - Focuses on finding and collecting energy sources
3. **⛏️ Mineral Collector** - Specializes in mining valuable minerals
4. **🧪 Scientist** - Seeks out science points for research advancement

### Resource Types

- **⚡ Energy** - Powers robots and station operations
- **⛏️ Minerals** - Used for creating new robots and station upgrades
- **🧪 Science Points** - Advance research and unlock new capabilities

### World Elements

- **🟫 Obstacles** - Impassable terrain that robots must navigate around
- **⬜ Empty Space** - Safe areas for robots to traverse
- **🏭 Station** - Central hub where robots refuel, unload resources, and share discoveries

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or higher
- Terminal with color support (most modern terminals)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd rusty-games
```

2. Build the project:
```bash
cargo build --release
```

3. Run the simulation:
```bash
cargo run
```

### Controls

- **Enter** - Start the simulation (on startup screen)
- **Q** - Quit the game during simulation
- The robots operate autonomously - no manual control needed!

## 🏗️ Project Structure

```
src/
├── main.rs         # Main game loop and initialization
├── robot.rs        # Robot AI, behaviors, and management
├── map.rs          # World generation and map management
├── station.rs      # Station logic and resource management
├── ui.rs           # Terminal UI and rendering
└── startup.rs      # Startup screen and intro
```

## 🧠 Technical Concepts

### Autonomous AI System

Each robot operates using a sophisticated AI system that includes:

- **State Machines**: Robots switch between exploring, returning to station, and being at station
- **A* Pathfinding**: Smart navigation using optimal pathfinding to avoid obstacles and find shortest routes
- **Directional Scoring**: Robots evaluate adjacent cells and choose the best direction based on their type
- **Resource Prioritization**: Different robot types have specialized collection preferences
- **Exploration Strategies**: Robots prefer unexplored areas and use "teleporting" to escape stuck situations
- **Swarm Coordination**: Robots avoid occupying the same cells and prevent clustering

### Map Generation

The world is procedurally generated using:

- **Perlin Noise**: Creates natural-looking terrain patterns
- **Resource Distribution**: Strategic placement of energy, minerals, and science points
- **Obstacle Placement**: Balanced challenge without blocking essential paths

### Resource Economy

The game features a balanced resource economy:

- **Energy Consumption**: Robots consume energy for movement and actions
- **Resource Collection**: Different resource types have varying rarity and value
- **Station Management**: Efficient resource allocation for maximum colony growth
- **Robot Creation Costs**: Balance between expansion and sustainability

### Performance Optimizations

- **Efficient A* Pathfinding**: Optimized pathfinding algorithm for intelligent robot navigation
- **Smart Rendering**: Only updates changed areas for smooth performance
- **Memory Management**: Careful resource allocation for large maps
- **Unstuck Mechanisms**: Robots can teleport to nearby unexplored areas when trapped

## 🎯 Game Strategies

### Early Game
- Focus on energy collection to keep robots active
- Spread explorers in different directions for maximum coverage
- Build a sustainable energy income before expanding

### Mid Game
- Balance exploration with resource collection
- Create specialized robots based on discovered resources
- Expand your robot fleet strategically

### Late Game
- Optimize robot distribution for maximum efficiency
- Focus on science point collection for advanced research
- Maintain large fleets for comprehensive world coverage

## 🔧 Customization

The game is highly configurable through code modifications:

- **Robot Behavior**: Modify AI parameters in `robot.rs`
- **World Generation**: Adjust map parameters in `map.rs`
- **Resource Economy**: Balance resource costs in `station.rs`
- **Visual Appearance**: Customize colors and symbols in `ui.rs`

## 📊 Dependencies

- **crossterm**: Cross-platform terminal manipulation
- **ratatui**: Modern terminal UI framework
- **noise**: Procedural noise generation for world creation
- **rand**: Random number generation for game mechanics

## 🐛 Known Issues & Future Enhancements

### Current Limitations
- Map size is limited by terminal dimensions
- No save/load functionality
- Limited robot specialization options

### Planned Features
- [ ] Research tree system
- [ ] Robot upgrades and modifications
- [ ] Multiple biome types
- [ ] Save/load game states
- [ ] Advanced swarm coordination algorithms
- [ ] Dynamic difficulty adjustment
