# ⚡ mcp-stama

<p align="center">
  <strong>The Ultra-Fast Swiss Army Knife MCP Server for AI Coding Agents</strong><br>
  <em>Blazingly fast, zero-dependency, single static Rust binary built for Cursor, Claude Desktop, and Windsurf.</em>
</p>

<p align="center">
  <a href="https://github.com/StamManif/mcp-stama/actions"><img src="https://img.shields.io/github/actions/workflow/status/StamManif/mcp-stama/release.yml?branch=main&style=flat-square&logo=github&label=build" alt="CI Status"></a>
  <a href="https://crates.io/crates/mcp-stama"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange?style=flat-square&logo=rust" alt="Crates.io"></a>
  <a href="https://github.com/StamManif/mcp-stama/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" alt="License"></a>
  <a href="#-benchmarks"><img src="https://img.shields.io/badge/p50_latency-327%C2%B5s-success?style=flat-square&logo=lightning" alt="Latency"></a>
</p>

---

> ⚡ **Why switch to `mcp-stama`?**
> Standard Node.js or Python MCP tools consume **200MB+ RAM** and take **1–3 seconds** just to wake up. `mcp-stama` runs natively in under **10MB RAM** with **sub-millisecond tool response times**, keeping your AI agent fast and your computer cool.

---

## 🚀 Performance Comparison

| Metric | Legacy MCP Servers (Node / Python) | ⚡ `mcp-stama` (Rust) |
| :--- | :--- | :--- |
| **Cold Startup Time** | 1,500ms – 3,000ms | **< 2ms** *(Instant)* |
| **p50 Execution Latency** | 150ms – 800ms | **300µs – 5ms** *(Sub-ms)* |
| **Memory Footprint (RSS)** | 180 MB – 350 MB | **< 10 MB** |
| **Dependencies** | 100+ `node_modules` / `pip` packages | **Zero external runtimes** |
| **Installation** | Requires Node.js / Python setup | **Single static binary** |

---

## 🔥 Built-in High-Performance Tools

Each tool in `mcp-stama` is engineered to give your AI agent deep local context **without wasting prompt tokens** or spawning heavy child processes:

* 🔎 **`fast_grep`**: Sub-millisecond file search and regex scanning using `ignore::WalkBuilder`. Respects `.gitignore` and `.ignore` automatically while skipping binary/hidden files.
* 📦 **`git_snapshot`**: Pure-Rust Git inspector powered natively by `gix` (`gitoxide`). Fetches HEAD commits, branch information, line deltas, and file status without calling the external `git` executable.
* 🐳 **`docker_watcher`**: Instant host and container diagnostics powered by `bollard` and `sysinfo`. Gives your AI instant visibility into running Docker containers, system memory, CPU cores, and mapped ports.
* ⚡ **`auto-configurator`**: Includes `--install-cursor` and `--install-claude` flags to automatically inject `mcp-stama` into your editor settings in under 2 seconds.
* 🛡️ **Pure Stdio JSON-RPC 2.0**: All diagnostic and logging outputs are routed strictly to `stderr`, keeping `stdout` 100% compliant for JSON-RPC frame protocol traffic.

---

## ⚡ Quick Start

### 1. One-Command Installation

#### macOS / Linux:
```bash
curl -fsSL [https://raw.githubusercontent.com/StamManif/mcp-stama/main/install.sh](https://raw.githubusercontent.com/StamManif/mcp-stama/main/install.sh) | sh
```

#### Windows (PowerShell):
```powershell
iwr -useb [https://raw.githubusercontent.com/StamManif/mcp-stama/main/install.ps1](https://raw.githubusercontent.com/StamManif/mcp-stama/main/install.ps1) | iex
```

#### Via Cargo (Rust developers):
```bash
cargo install mcp-stama
```

---

### 2. Auto-Connect to Your AI Editor

Skip manual JSON editing! Let `mcp-stama` configure your AI client automatically:

##Auto-configure for Cursor

```bash
mcp-stama --install-cursor
```
#Auto-configure for Claude Desktop

```bash
mcp-stama --install-claude
```

---

### 3. Manual Configuration (Optional)

If you prefer adding it manually to your settings file:

#### Cursor (`~/.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "mcp-stama": {
      "command": "mcp-stama",
      "args": []
    }
  }
}
```

#### Claude Desktop (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "mcp-stama": {
      "command": "mcp-stama",
      "args": []
    }
  }
}
```

---

## 📊 Benchmarks

Run the embedded benchmark suite on your own machine at any time with `mcp-stama --benchmark`:

| Tool Name | Invocations | p50 Latency | p99 Latency | Memory Footprint (RSS) |
| :--- | :--- | :--- | :--- | :--- |
| **`docker_watcher`** | 100 | **327 µs** | **1.66 ms** | `< 10 MB` |
| **`git_snapshot`** | 100 | **462 µs** | **1.36 ms** | `< 10 MB` |
| **`fast_grep`** | 100 | **5.05 ms** | **8.72 ms** | `< 10 MB` |

---

## 🛠️ Architecture

```
                 +-------------------------------+
                 |  AI Client (Cursor / Claude)  |
                 +---------------+---------------+
                                 |
                          stdio (JSON-RPC)
                                 |
                 +---------------v---------------+
                 |        StdioTransport         |
                 +---------------+---------------+
                                 |
                     JsonRpcRequest / Response
                                 |
                 +---------------v---------------+
                 |          ToolRegistry         |
                 +---------------+---------------+
                                 |
         +-----------------------+-----------------------+
         |                       |                       |
+--------v--------+     +--------v--------+     +--------v--------+
|    fast_grep    |     |  git_snapshot   |     | docker_watcher  |
| (ignore/regex)  |     |   (gix/rust)    |     | (bollard/sys)   |
+-----------------+     +-----------------+     +-----------------+
```

---

## 🛠️ Building from Source

```bash
# Clone the repository
git clone [https://github.com/StamManif/mcp-stama.git](https://github.com/StamManif/mcp-stama.git)
cd mcp-stama

# Run test suite
cargo test

# Build release binary
cargo build --release

# Run internal micro-benchmarks
./target/release/mcp-stama --benchmark
```

---

## 📄 License

Licensed under the **Apache License, Version 2.0** ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).
