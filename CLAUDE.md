# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an Alfred workflow written in Rust that integrates with the Zed editor. The workflow provides four main commands:
- `zf`: Search files and open with Zed
- `zr`: Open recent projects (reads from Zed's SQLite database)
- `zd`: Open project directories from a configured folder (configurable via Alfred UI)
- `ze`: Lookup Zed extensions

## Build and Development Commands

### Building the Project
```bash
cargo build --release
```

### Running/Testing
```bash
cargo run
cargo run "search_query"
```

### Development Build
```bash
cargo build
```

## Code Architecture

### Core Components
- **main.rs**: Single-file application containing all logic
- **Workspace struct**: Represents a workspace from Zed's database
- **Item struct**: Alfred workflow item format for JSON output
- **Icon struct**: File icon configuration for Alfred items

### Key Functionality
- **Database Integration**: Reads from Zed's SQLite database at `~/.config/Zed/db/0-stable/db.sqlite`
- **Directory Listing**: Lists subdirectories from configured project directories (via `--dirs` flag)
- **Query Processing**: Filters results based on command-line arguments
- **Alfred Output**: Serializes results as JSON for Alfred consumption

### CLI Usage
```bash
./alfred-zed [query]           # List recent projects (zr command)
./alfred-zed --dirs [query]    # List project directories (zd command)
```

### Data Flow
**Recent Projects (default mode):**
1. Connects to Zed's SQLite database
2. Queries workspaces table for recent projects
3. Converts workspace data to Alfred item format
4. Filters results based on user query
5. Outputs JSON to stdout for Alfred

**Project Directories (`--dirs` mode):**
1. Reads `projects_directories` environment variable
2. Lists all subdirectories from each configured directory
3. Filters and sorts results
4. Outputs JSON to stdout for Alfred

## Alfred Workflow Configuration

The workflow defines four filters in `workflow/info.plist`:
- **zf**: File search using Alfred's file filter
- **zr**: Recent projects using `alfred-zed` binary (default mode)
- **zd**: Project directories using `alfred-zed --dirs` (reads from `projects_directories` env var)
- **ze**: Extensions lookup using `jq` to parse Zed's extensions index

### Workflow Variables
The workflow uses user-configurable variables (set via Alfred's workflow configuration UI):
- **projects_directories**: List of directories containing project folders for the `zd` command (one per line)

## Dependencies

External dependencies required:
- **jq**: Required for the `ze` command (extensions lookup)
- **Zed editor**: Must be installed for the workflow to function

Rust crate dependencies:
- `dirs`: For accessing config directories
- `serde`: JSON serialization
- `serde_json`: JSON handling
- `sqlite`: Database access

## Release Configuration

The project uses aggressive optimization settings in `Cargo.toml`:
- Link-time optimization enabled
- Minimal binary size (`opt-level = 'z'`)
- Symbol stripping for smaller binaries
- Panic abort for reduced size