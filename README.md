<h1>Cavira</h1>

<center>

<img src="./logo.png" width=50%>

**Cavira** — process execution analysis tool

</center>

Cavira is a lightweight CLI tool for profiling process execution.\
It measures CPU usage, memory consumption, and runtime — either by spawning a new command or attaching to an existing process.\
Results can be saved, compared, and reviewed at any time.

## 1. Usage

```
cavira <COMMAND> [OPTIONS]
```

### 1.1. Global Options

| Flag | Description |
|------|-------------|
| `--no-color` | Disable colored output |
| `--quiet` | Suppress non-essential output |
| `--config <path>` | Path to config file |

---

### 1.2. `run` — Track a command's execution

```
cavira run <COMMAND> [OPTIONS]
```

Spawns the given command and profiles its resource usage until it exits or the timeout is reached.

| Argument / Option | Description |
|-------------------|-------------|
| `<COMMAND>` | Command to execute |
| `-i, --interval <ms>` | Sampling interval in milliseconds (default: `100`) |
| `-t, --timeout <time>` | Maximum execution time (e.g. `2s`, `500ms`) |
| `--json <path>` | Save results to a JSON file at the given path |
| `--silent` | Suppress the command's stdout |

---

### 1.3. `pid` — Attach to a running process

```
cavira pid <PID> [OPTIONS]
```

Attaches to an already-running process by PID and profiles it.

| Argument / Option | Description |
|-------------------|-------------|
| `<PID>` | Target process ID |
| `-i, --interval <ms>` | Sampling interval in milliseconds (default: `100`) |
| `--duration <time>` | How long to track the process (e.g. `10m`, `30s`) |
| `--json <path>` | Save results to a JSON file at the given path |

---

### 1.4. `compare` — Compare two run results

```
cavira compare <RUN1> <RUN2> [OPTIONS]
```

Compares two recorded runs side by side.

| Argument / Option | Description |
|-------------------|-------------|
| `<RUN1>` | First run — JSON file path or run ID |
| `<RUN2>` | Second run — JSON file path or run ID |
| `--metric <type>` | Metric to compare: `cpu`, `mem`, or `time` |

---

### 1.5. `history` — List past runs

```
cavira history [OPTIONS]
```

Displays the recorded run history stored locally.

| Option | Description |
|--------|-------------|
| `-n, --limit <N>` | Show the most recent N entries |
| `--filter <keyword>` | Filter entries by command name or keyword |
| `--json` | Output results as JSON |

---

### 1.6. `inspect` — Show details of a specific run

```
cavira inspect <RUN_ID>
```

Prints full profiling data for a single recorded run.

| Argument | Description |
|----------|-------------|
| `<RUN_ID>` | ID of the run to inspect |

---

### 1.7. `live` — Real-time monitoring

```
cavira live <COMMAND>
```

Runs a command and displays live resource usage alongside its output in the terminal.

---

## 2. Features

### 2.1. CLI

- [x] Root command entry point (`cavira`)
- [x] Global option: `--no-color`
- [x] Global option: `--quiet`
- [x] Global option: `--config <path>`

### 2.2. `run`

- [x] Spawn subprocess and execute command
- [x] Sample CPU & memory usage at configurable interval (`-i`)
- [x] Enforce maximum execution time (`-t / --timeout`)
- [x] Save profiling results to JSON (`--json`)
- [x] Suppress command stdout (`--silent`)

### 2.3. `pid`

- [x] Attach to an existing process by PID
- [x] Sample CPU & memory usage at configurable interval (`-i`)
- [x] Stop tracking after given duration (`--duration`)
- [x] Save profiling results to JSON (`--json`)

### 2.4. `compare`

- [x] Load two run results from JSON files or run IDs
- [x] Compare by CPU usage (`--metric cpu`)
- [x] Compare by memory usage (`--metric mem`)
- [x] Compare by execution time (`--metric time`)
- [x] Render side-by-side diff output

### 2.5. `history`

- [x] Persist run records to local storage
- [x] List all recorded runs
- [x] Limit output to most recent N entries (`-n / --limit`)
- [x] Filter entries by keyword (`--filter`)
- [x] Output history as JSON (`--json`)

### 2.6. `inspect`

- [x] Look up a run record by ID
- [x] Display full profiling data for the run

### 2.7. `live`

- [x] Run command with real-time resource overlay
- [x] Update CPU & memory display on each sample tick
