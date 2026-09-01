# pager

A multi-column terminal pager for git diffs. Pipe any diff into it and it lays the output across N columns that all scroll together — like reading a newspaper, where column 1 ends and column 2 picks up from there.

```
git diff | pager 3
```

```
┌─ 1 ──────────────────┐┌─ 2 ──────────────────┐┌─ 3 ──────────────────┐
│ diff --git a/foo ...  ││ -    let x = 1;       ││ +    let x = 2;       │
│ index 1234..abcd      ││ +    let x = 2;       ││      let y = 3;       │
│ --- a/foo.rs          ││      let y = 3;       ││      let z = x + y;   │
│ +++ b/foo.rs          ││      let z = x + y;   ││  }                    │
│ @@ -1,5 +1,5 @@      ││  }                    ││                       │
└───────────────────────┘└───────────────────────┘└───────────────────────┘
 lines 1-90/210  43%  cols:3   j/k scroll · q quit
```

## Install

```bash
cargo build --release
# binary is at target/release/pager.exe (Windows) or target/release/pager (Unix)
```

### Windows: install from `C:\dev\pager`

Build the release executable:

```powershell
cargo build --release --manifest-path C:\dev\pager\Cargo.toml
```

Add its directory to the beginning of your user `PATH`:

```powershell
$pagerBin = "C:\dev\pager\target\release"
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$entries = @($userPath -split ";" | Where-Object { $_ -and $_ -ne $pagerBin })
$newPath = (@($pagerBin) + $entries) -join ";"
[Environment]::SetEnvironmentVariable("Path", $newPath, "User")
```

Open a new terminal and verify the installed command:

```powershell
Get-Command pager
git diff | pager
```

After changing the app, update the executable by running the build command again.

## Usage

```bash
# 2 columns (default)
git diff | pager

# 3 columns
git diff | pager 3
git diff | pager --columns 3
git diff | pager -n 3

# pipe through delta for syntax highlighting (inline)
git diff | pager --delta 3

# pipe through delta side-by-side — each pager column shows a full old|new panel
git diff | pager --delta-sbs 2

# pipe through diff-so-fancy (requires: npm install -g diff-so-fancy)
git diff | pager --fancy 2
```

Works with anything that produces diff-like output:

```bash
git show abc1234 | pager 3
git diff HEAD~5 | pager --delta 2
git log --stat | pager 3
```

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | scroll down one line |
| `k` / `↑` | scroll up one line |
| `d` / `u` | half-page down / up |
| `f` / `b` / PgDn / PgUp | full page down / up |
| `g` / Home | jump to top |
| `G` / End | jump to bottom |
| `+` / `=` | add a column |
| `-` | remove a column |
| `q` / Ctrl-C | quit |

Column count can also be changed live with `+`/`-` without restarting.

## Code structure

```
src/
  main.rs   — arg parsing, stdin reading, formatter dispatch, TUI lifecycle, key handling
  app.rs    — scroll state and column slicing logic
  ui.rs     — ratatui rendering: column layout, ANSI parser, status bar
```

**`app.rs`** owns the data model: a flat `Vec<String>` of lines (with ANSI codes intact) and a single `offset: usize`. `column_lines(col, page_height)` slices out the right range for each column — column N shows lines `[offset + N*page_height .. offset + (N+1)*page_height]`.

**`ui.rs`** renders using ratatui. It includes a hand-written ANSI SGR parser (`parse_ansi`) that converts escape sequences into ratatui `Span`s with proper styles. This avoids the `ansi-to-tui` crate which had ratatui version conflicts. Supports standard colors (30–37, 90–97), 256-color (`38;5;N`), RGB (`38;2;r;g;b`), and common modifiers (bold, italic, dim, underline, etc.).

**`main.rs`** handles the three formatter modes via a `Formatter` enum:
- `None` — raw stdin passed directly
- `Delta { side_by_side }` — spawns `delta --pager never [--side-by-side --width N]`
- `Fancy` — spawns `diff-so-fancy`

For `--delta-sbs`, the terminal width is queried *before* entering raw mode so we can pass the correct per-column width to delta via `--width`.

## Dependencies

- [`ratatui`](https://crates.io/crates/ratatui) — TUI rendering
- [`crossterm`](https://crates.io/crates/crossterm) — terminal input/output, raw mode

No other runtime dependencies. ANSI parsing is done inline rather than via a crate to avoid version conflicts.

## Optional external tools

| Flag | Tool | Install |
|------|------|---------|
| `--delta` / `--delta-sbs` | [delta](https://github.com/dandavison/delta) | `cargo install git-delta` |
| `--fancy` | [diff-so-fancy](https://github.com/so-fancy/diff-so-fancy) | `npm install -g diff-so-fancy` |
