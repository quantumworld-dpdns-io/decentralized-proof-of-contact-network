# MCP Tools Reference

The `poi-mcp-server` crate implements the Model Context Protocol (MCP), enabling AI agents (Claude Code, etc.) to interact with the proof-of-contact network.

## Overview

MCP provides a standardized interface for AI tools. The POI MCP server exposes network operations as callable tools that AI agents can discover and invoke.

## Server Setup

```bash
# Build the MCP server
cargo build -p poi-mcp-server

# Run with default settings (connects to localhost:3000)
cargo run -p poi-mcp-server

# Run with custom API URL
cargo run -p poi-mcp-server -- --api-url http://localhost:3000
```

## Configuration (Claude Code)

Add to `.claude/settings.json`:

```json
{
  "mcpServers": {
    "poi-network": {
      "command": "poi-mcp-server",
      "args": ["--api-url", "http://localhost:3000"]
    }
  }
}
```

## Available Tools

### `get_node_info`

Returns local node information.

**Input**: None

**Output**: Node ID, public key, version, uptime, peer count.

### `list_peers`

Returns connected peer list.

**Input**: None

**Output**: Array of peer info (ID, address, connection time, reputation).

### `create_proof`

Creates a new proof-of-contact.

**Input**:
- `target_node` (string) — Target node ID
- `window_id` (string) — Orbital window UUID
- `purpose` (string, optional) — Proof purpose description

**Output**: Full `ContactProof` object.

### `verify_proof`

Verifies a proof's cryptographic integrity.

**Input**:
- `proof_id` (string) — UUID of the proof to verify

**Output**: `FullVerificationReport` with signature, timestamp, window, and chain checks.

### `get_proof`

Retrieves a specific proof by ID.

**Input**:
- `proof_id` (string) — UUID of the proof

**Output**: Full `ContactProof` object.

### `list_proofs`

Lists proofs with pagination.

**Input**:
- `limit` (integer, optional, default 50)
- `offset` (integer, optional, default 0)

**Output**: Array of `ContactProof` objects.

### `search_proofs`

Semantic search across proofs.

**Input**:
- `query` (string) — Natural language search query
- `limit` (integer, optional, default 10)

**Output**: Array of matching proofs with relevance scores.

### `list_windows`

Lists orbital windows.

**Input**:
- `active_only` (boolean, optional, default false)

**Output**: Array of `OrbitalWindow` objects.

### `create_window`

Creates a new orbital window.

**Input**:
- `start_time` (string) — ISO 8601 start time
- `end_time` (string) — ISO 8601 end time
- `window_type` (string, optional, default "standard")

**Output**: Created `OrbitalWindow` object.

### `ai_query`

Sends a natural language query to the AI engine.

**Input**:
- `question` (string) — Natural language question

**Output**: Text response from the AI model.

### `detect_anomalies`

Detects anomalous patterns in the network.

**Input**: None

**Output**: Array of detected anomalies with severity, type, and description.

### `get_network_stats`

Returns network-wide statistics.

**Input**: None

**Output**: Network statistics (total proofs, active nodes, etc.).

## Example AI Agent Interactions

### With Claude Code

When connected via MCP, you can ask Claude:

> "Check the health of my POI network node"

Claude will call `get_node_info` and `list_peers` to gather data.

> "Create a proof that I contacted node beta-node in the current orbital window"

Claude will call `list_windows` (with `active_only=true`), then `create_proof` with the active window.

> "Are there any anomalies in the network?"

Claude will call `detect_anomalies` and summarize the results.

> "Verify the latest proof and tell me if it's valid"

Claude will call `list_proofs` to get the latest, then `verify_proof` on it.

## Protocol Details

MCP uses JSON-RPC 2.0 for tool call/result exchange. Tools are discovered via the `tools/list` method and invoked via `tools/call`.

All POI MCP tool responses are JSON objects. Errors are returned as JSON-RPC error objects with descriptive messages.

## Security

- The MCP server uses the same API key authentication as the REST API
- No additional authentication is added at the MCP layer
- The MCP server should only be exposed on localhost or behind a secure reverse proxy
- Sensitive operations (proof creation, window management) require API key auth when configured
