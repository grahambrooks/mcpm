# MCPM - Model Context Protocol Package Manager

A terminal user interface (TUI) for managing [Model Context Protocol (MCP)](https://modelcontextprotocol.io) servers across different IDEs.

## Features

- **Interactive TUI**: Browse and search MCP servers from the official registry
- **Fuzzy Search**: Quickly find servers with fuzzy search filtering
- **Direct Server Access**: Jump directly to a specific server with `--server` flag
- **Registry Integration**: Fetches the complete MCP registry with non-blocking loading
- **IDE Configuration Management**: View server details and installation instructions

## Installation

### From Source

```bash
git clone <repository-url>
cd mcpm
cargo build --release
```

The binary will be available at `target/release/mcpm`.

## Usage

### Basic Usage

Launch the interactive TUI:

```bash
mcpm
```

### Command-Line Options

```bash
# Jump directly to a specific server
mcpm --server <server-name>

# Enable debug logging
mcpm --debug
```

### TUI Navigation

- `↑/↓` or `j/k`: Navigate through server list
- `/`: Start fuzzy search
- `Enter`: View server details
- `Esc`: Go back / Clear search
- `q`: Quit

## Architecture

The project is structured into several key modules:

- **app**: Main application state and event handling
- **model**: Data models for MCP servers and state management
- **registry**: Integration with the MCP registry API
- **services**: Background services for async operations
- **view**: TUI rendering with [ratatui](https://ratatui.rs)
- **ide**: IDE-specific configuration handling

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Code Quality

```bash
# Run clippy for linting
cargo clippy

# Format code
cargo fmt
```

## Requirements

- Rust 1.70 or later
- Terminal with Unicode support for best display

## License

[Add your license here]

## Contributing

[Add contributing guidelines here]
