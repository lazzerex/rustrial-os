# Shell/Command Interpreter

## Overview

RustrialOS now includes a fully functional command-line shell interface that provides a traditional terminal experience alongside the graphical desktop environment.

## Features

- **Command Parsing**: Parse and execute commands with arguments
- **Command History**: Navigate previous commands using arrow keys (Up/Down)
- **Interactive Input**: Full keyboard input with backspace support
- **Filesystem Integration**: Access and manipulate the virtual filesystem
- **Script Execution**: Run RustrialScript files directly from the shell
- **Color Customization**: Change terminal colors on the fly
- **Current Directory**: Navigate the filesystem with cd/pwd commands

## Available Commands

### File Operations
- `ls [path]` - List files and directories in current or specified path
- `cat <file>` - Display file contents (text or hex dump for binary)
- `mkdir <dir>` - Create a new directory
- `touch <file>` - Create an empty file
- `cd <dir>` - Change current directory
- `pwd` - Print working directory

### Script Execution
- `run <script>` - Execute a RustrialScript file
  - Automatically searches `/scripts` directory
  - Can use relative or absolute paths
  - Lists available scripts if no argument provided

### System Commands
- `help` - Display all available commands and usage
- `clear` / `cls` - Clear the screen
- `echo <text>` - Print text to the terminal
- `color <fg> <bg>` - Change text colors (0-15)
- `exit` / `quit` - Return to desktop environment

## Usage

### Launching the Shell

The shell can be accessed in two ways:

1. **From Desktop**: Double-click the "Shell" icon (displays `[>_]`)
2. **From Code**: Call `rustrial_os::rustrial_menu::menu_system::launch_shell().await`

### Examples

```bash
# List all files in the current directory
rustrial> ls

# Navigate to scripts directory
rustrial> cd /scripts

# List available scripts
rustrial> ls

# Run a script
rustrial> run fibonacci.rscript

# Display script source code
rustrial> cat fibonacci.rscript

# Create a new directory
rustrial> mkdir /myfiles

# Create a new file
rustrial> touch /myfiles/test.txt

# Change colors (green on black)
rustrial> color 10 0

# Clear the screen
rustrial> clear

# Return to desktop
rustrial> exit
```

## Color Codes

Use with the `color <foreground> <background>` command:

| Code | Color       | Code | Color       |
|------|-------------|------|-------------|
| 0    | Black       | 8    | DarkGray    |
| 1    | Blue        | 9    | LightBlue   |
| 2    | Green       | 10   | LightGreen  |
| 3    | Cyan        | 11   | LightCyan   |
| 4    | Red         | 12   | LightRed    |
| 5    | Magenta     | 13   | Pink        |
| 6    | Brown       | 14   | Yellow      |
| 7    | LightGray   | 15   | White       |

## Architecture

### Implementation Details

The shell is implemented in `src/shell.rs` and provides:

- **Async Input Handling**: Uses the existing keyboard scancode stream
- **History Buffer**: Stores up to 50 previous commands
- **Path Resolution**: Supports absolute paths, relative paths, `.` and `..`
- **VFS Integration**: Direct integration with the RamFS filesystem
- **Script Runner Integration**: Calls `rustrial_script::run()` for script execution

### Key Components

```rust
pub struct Shell {
    input_buffer: String,      // Current command being typed
    history: Vec<String>,       // Command history
    history_index: usize,       // Current position in history
    current_dir: String,        // Current working directory
    foreground_color: Color,    // Terminal foreground color
    background_color: Color,    // Terminal background color
}
```

### Integration Points

1. **Desktop Integration**: Shell icon in desktop environment (`src/desktop.rs`)
2. **Menu System**: Launch function in menu system (`src/rustrial_menu/menu_system/mod.rs`)
3. **Filesystem**: Uses VFS abstractions (`src/fs/`)
4. **Script Engine**: Executes RustrialScript files (`src/rustrial_script/`)

## Future Enhancements

Potential improvements for the shell:

- [ ] Tab completion for commands and filenames
- [ ] Command aliases
- [ ] Environment variables
- [ ] Pipes and redirection (`|`, `>`, `<`)
- [ ] Job control (background processes)
- [ ] Built-in text editor
- [ ] More filesystem commands (rm, mv, cp, rmdir)
- [ ] Command output paging
- [ ] Script arguments passing
- [ ] Command scripting (.sh files)

## Technical Notes

- The shell is fully asynchronous and non-blocking
- Input handling uses PC keyboard scancodes
- All filesystem operations use the VFS abstraction layer
- Path resolution handles both absolute and relative paths
- Binary files display a hex dump instead of raw bytes
- The shell properly restores colors after returning to desktop
