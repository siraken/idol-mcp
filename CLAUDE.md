# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of a Model Context Protocol (MCP) server that provides information about Japanese idol groups. Currently supports KAWAII LAB. groups with plans to expand to other idol groups and agencies. The server exposes a `members` tool that allows fuzzy searching for idol group members by group name, nickname, or member names.

## Build and Run Commands

```bash
# Build debug version
cargo build

# Build optimized release version
cargo build --release

# Run in development
cargo run

# Run release binary directly
./target/release/idol-mcp
```

## MCP Integration

The server is designed to be used with Claude Desktop via MCP. Configuration in `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "idol-lab": {
      "command": "/path/to/target/release/idol-mcp"
    }
  }
}
```

## Architecture

The codebase follows a modular structure:

- **`main.rs`**: MCP server implementation using `rmcp` crate with macro-based tool routing
- **`types.rs`**: Core data structures (`KawaiiLabGroup`, `KawaiiLabMember`) that can be extended for other agencies
- **`data.rs`**: Hardcoded idol group data, currently containing KAWAII LAB. groups (FRUITS ZIPPER, CANDY TUNE, SWEET STEADY, CUTIE STREET)
- **`tools.rs`**: Unused legacy file (can be removed)

### Key Implementation Details

- Uses `rmcp` 0.3.2 with macro-based approach (`#[tool_router]`, `#[tool_handler]`, `#[tool]`)
- Fuzzy search powered by `fuzzy-matcher` crate with `SkimMatcherV2`
- Tracing logs directed to stderr to avoid interfering with JSON-RPC communication on stdout
- Server communicates via stdio transport for MCP compatibility

### Search Functionality

The `members` tool searches across:

- Group names (official and common names)
- Member names (full names and nicknames)
- Uses fuzzy matching with scoring to return best match

## Expanding to Other Idol Groups

The current architecture is designed to be extensible:

### Adding New Agencies/Groups

1. **Data Structure Extension**: 
   - Current types are KAWAII LAB.-specific (`KawaiiLabGroup`, `KawaiiLabMember`)
   - For other agencies, consider creating generic types or agency-specific modules
   - Example approach: `src/agencies/` with separate modules for each agency

2. **Data Organization**:
   - Current `data.rs` contains hardcoded KAWAII LAB. data
   - For scalability, consider moving to JSON files or database
   - Maintain fuzzy search capabilities across all groups

3. **Tool Enhancement**:
   - Current `members` tool searches only KAWAII LAB. groups
   - May need agency-specific tools or enhanced search with agency filtering
   - Consider adding tools for schedules, discography, etc.

### Implementation Suggestions

```rust
// Example: Generic group structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdolGroup {
    pub agency: String,
    pub name: String,
    pub name_katakana: String,
    pub common_name: String,
    pub debut_date: Option<String>,
    pub members: Vec<IdolMember>,
}
```

## Important Notes

- **Logging**: All tracing output must go to stderr, not stdout (MCP uses stdout for JSON-RPC)
- **Binary Path**: Claude Desktop requires the full absolute path to the release binary
- **Error Handling**: Server handles broken pipes gracefully when Claude Desktop disconnects
