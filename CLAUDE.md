# hangrier_games

## Codebase Navigation — MUST USE indxr MCP tools

An MCP server called `indxr` is available. **Always use indxr tools before the Read tool.** Do NOT read full source files as a first step — use the MCP tools to explore, then read only what you need.

### Token savings reference

The MCP server defaults to **3 compound tools** (`find`, `summarize`, `read`). All 26 tools (3 compound + 23 granular) are available with `--all-tools`.

| Action | Approx tokens | When to use |
|--------|--------------|-------------|
| `find(query)` | ~100-400 | Find files/symbols by concept, name, callers, or signature pattern |
| `summarize(path)` | ~200-600 | Understand a file, batch of files, or symbol without reading source |
| `read(path, symbol?)` | ~50-300 | Read one function/struct. Supports `symbols` array and `collapse`. |
| `Read` (full file) | **500-10000+** | ONLY when editing or need exact formatting |

**Typical exploration: ~500 tokens vs ~3000+ for reading a full file (6x reduction).**

### Exploration workflow (follow this order)

The default 3 compound tools cover the most common exploration patterns:

1. `find(query)` — find files/symbols by concept, partial name, or type pattern. **Start here when you know what you're looking for but not where it is.**
   - Default mode (`relevant`): multi-signal relevance search across paths, names, signatures, and docs. Supports `kind` filter.
   - `mode: "symbol"`: find declarations by name (case-insensitive substring).
   - `mode: "callers"`: find who references a symbol (imports + signatures).
   - `mode: "signature"`: find functions by signature pattern (e.g., `"-> Result<"`).
2. `summarize(path)` — understand files and symbols without reading source code.
   - File path (e.g., `"src/main.rs"`): complete file overview (declarations, imports, counts).
   - Glob pattern (e.g., `"src/mcp/*.rs"`): batch summaries for multiple files.
   - Symbol name (no `/`, e.g., `"Cache"`): full interface details (signature, doc comment, relationships).
   - `scope: "public"`: show only public API surface.
3. `read(path, symbol?)` — read source code by **symbol name** or explicit line range. Cap: 200 lines. Use `symbols` array to read multiple in one call (500 line cap). Use `collapse: true` to fold nested bodies.

With `--all-tools`, all 23 granular tools are also exposed. Key granular tools:
- `get_tree` — directory/file layout
- `get_file_context` — reverse dependencies and related files
- `get_token_estimate` — check token cost before reading
- `get_diff_summary` — structural changes since a git ref or GitHub PR
- `get_hotspots` — most complex functions ranked by composite score
- `get_health` — codebase health summary
- `get_type_flow` — track where a type flows across function boundaries
- `regenerate_index` — re-index after code changes

If built with `--features wiki` and a wiki exists (`indxr wiki generate`):
- `wiki_search(query)` — search the codebase knowledge wiki by keyword. **Use first for understanding modules/architecture.**
- `wiki_read(page)` — read a wiki page by ID (e.g. `"architecture"`, `"mod-mcp"`)
- `wiki_status()` — check wiki health: page count, staleness, coverage

### When to use the Read tool instead
- You need to **edit** a file (Read is required before Edit)
- You need exact formatting/whitespace that `read` doesn't preserve
- The file is not a source file (e.g., config files, documentation)

### DO NOT
- Read full source files just to understand what's in them — use `summarize(path)`
- Read full source files to review code — use `summarize(path)` to triage, then `read(path, symbol)` on specific symbols
- Dump all files into context — use MCP tools to be surgical
- Use `git diff` to understand changes — use `get_diff_summary` instead (requires `--all-tools`)

### After making code changes
Run `regenerate_index` to keep INDEX.md current.
