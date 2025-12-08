//! Shell/Command Interpreter for RustrialOS
//! 
//! A simple command-line interface with support for:
//! - File operations (ls, cat, mkdir, touch)
//! - Script execution (run)
//! - System commands (help, clear, echo, exit)
//! - Color customization

use crate::{print, println};
use crate::task::keyboard;
use crate::vga_buffer::{Color, WRITER};
use crate::fs::{FileSystem, FileType};
use crate::rustrial_script;
use alloc::{string::String, vec::Vec, format};
use alloc::string::ToString;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use futures_util::stream::StreamExt;

const PROMPT: &str = "rustrial> ";
const MAX_HISTORY: usize = 50;

pub struct Shell {
    input_buffer: String,
    history: Vec<String>,
    history_index: usize,
    current_dir: String,
    foreground_color: Color,
    background_color: Color,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            input_buffer: String::new(),
            history: Vec::new(),
            history_index: 0,
            current_dir: String::from("/"),
            foreground_color: Color::LightGreen,
            background_color: Color::Black,
        }
    }

    /// Run the shell interactively
    pub async fn run(&mut self) {
        self.print_welcome();
        
        let mut scancodes = keyboard::ScancodeStream::new();
        let mut kb = Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        );

        loop {
            self.print_prompt();
            
            // Read a line of input
            if let Some(line) = self.read_line(&mut scancodes, &mut kb).await {
                if !line.is_empty() {
                    // Add to history
                    if self.history.len() >= MAX_HISTORY {
                        self.history.remove(0);
                    }
                    self.history.push(line.clone());
                    self.history_index = self.history.len();
                    
                    // Execute command
                    if self.execute_command(&line).await {
                        break; // Exit command was issued
                    }
                }
            }
        }
    }

    fn print_welcome(&self) {
        println!("\n╔════════════════════════════════════════════════════════════════════╗");
        println!("║              Welcome to RustrialOS Shell v0.1                      ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!("║  Type 'help' for available commands                               ║");
        println!("║  Type 'exit' to return to desktop                                 ║");
        println!("╚════════════════════════════════════════════════════════════════════╝\n");
    }

    fn print_prompt(&self) {
        // Set prompt color
        WRITER.lock().set_color(self.foreground_color, self.background_color);
        print!("{}{} ", self.current_dir, PROMPT);
    }

    /// Read a line of input from the keyboard
    async fn read_line(
        &mut self,
        scancodes: &mut keyboard::ScancodeStream,
        kb: &mut Keyboard<layouts::Us104Key, ScancodeSet1>,
    ) -> Option<String> {
        self.input_buffer.clear();
        let saved_buffer = String::new();

        loop {
            if let Some(scancode) = scancodes.next().await {
                if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                    if let Some(key) = kb.process_keyevent(key_event) {
                        match key {
                            DecodedKey::Unicode(character) => {
                                match character {
                                    '\n' => {
                                        println!();
                                        return Some(self.input_buffer.clone());
                                    }
                                    '\u{0008}' => {
                                        // Backspace
                                        if !self.input_buffer.is_empty() {
                                            self.input_buffer.pop();
                                            print!("\u{0008} \u{0008}");
                                        }
                                    }
                                    _ => {
                                        if !character.is_control() {
                                            self.input_buffer.push(character);
                                            print!("{}", character);
                                        }
                                    }
                                }
                            }
                            DecodedKey::RawKey(code) => {
                                match code {
                                    KeyCode::ArrowUp => {
                                        if self.history_index > 0 {
                                            self.history_index -= 1;
                                            self.load_history_entry();
                                        }
                                    }
                                    KeyCode::ArrowDown => {
                                        if self.history_index < self.history.len() {
                                            self.history_index += 1;
                                            self.load_history_entry();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_history_entry(&mut self) {
        // Clear current line
        for _ in 0..self.input_buffer.len() {
            print!("\u{0008} \u{0008}");
        }
        
        // Load history entry or clear
        if self.history_index < self.history.len() {
            self.input_buffer = self.history[self.history_index].clone();
            print!("{}", self.input_buffer);
        } else {
            self.input_buffer.clear();
        }
    }

    /// Execute a command and return true if shell should exit
    async fn execute_command(&mut self, line: &str) -> bool {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let command = parts[0];
        let args = &parts[1..];

        match command {
            "help" => self.cmd_help(),
            "clear" | "cls" => self.cmd_clear(),
            "echo" => self.cmd_echo(args),
            "ls" | "dir" => self.cmd_ls(args),
            "cat" => self.cmd_cat(args),
            "mkdir" => self.cmd_mkdir(args),
            "touch" => self.cmd_touch(args),
            "run" => self.cmd_run(args).await,
            "cd" => self.cmd_cd(args),
            "pwd" => self.cmd_pwd(),
            "color" => self.cmd_color(args),
            "exit" | "quit" => return true,
            _ => {
                println!("Unknown command: '{}'. Type 'help' for available commands.", command);
            }
        }

        false
    }

    fn cmd_help(&self) {
        println!("\nAvailable Commands:");
        println!("  help              - Show this help message");
        println!("  clear, cls        - Clear the screen");
        println!("  echo <text>       - Print text to the screen");
        println!("  ls [path]         - List files and directories");
        println!("  cat <file>        - Display file contents");
        println!("  mkdir <dir>       - Create a directory");
        println!("  touch <file>      - Create an empty file");
        println!("  run <script>      - Execute a RustrialScript file");
        println!("  cd <dir>          - Change current directory");
        println!("  pwd               - Print working directory");
        println!("  color <fg> <bg>   - Change text color (0-15)");
        println!("  exit, quit        - Return to desktop");
        println!("\nColors: 0=Black, 1=Blue, 2=Green, 3=Cyan, 4=Red, 5=Magenta, 6=Brown,");
        println!("        7=LightGray, 8=DarkGray, 9=LightBlue, 10=LightGreen, 11=LightCyan,");
        println!("        12=LightRed, 13=Pink, 14=Yellow, 15=White\n");
    }

    fn cmd_clear(&self) {
        WRITER.lock().clear_screen();
    }

    fn cmd_echo(&self, args: &[&str]) {
        println!("{}", args.join(" "));
    }

    fn cmd_ls(&self, args: &[&str]) {
        let path = if args.is_empty() {
            self.current_dir.as_str()
        } else {
            args[0]
        };

        let full_path = self.resolve_path(path);

        if let Some(fs) = crate::fs::root_fs() {
            let fs = fs.lock();
            match fs.read_dir(&full_path) {
                Ok(entries) => {
                    println!("\nDirectory: {}", full_path);
                    println!("─────────────────────────────────────");
                    
                    if entries.is_empty() {
                        println!("  (empty)");
                    } else {
                        for entry in entries {
                            let type_str = match entry.file_type {
                                FileType::File => "[FILE]",
                                FileType::Directory => "[DIR] ",
                            };
                            let size_str = if entry.file_type == FileType::File {
                                format!(" ({} bytes)", entry.size)
                            } else {
                                String::new()
                            };
                            println!("  {} {}{}", type_str, entry.name, size_str);
                        }
                    }
                    println!();
                }
                Err(e) => {
                    println!("Error listing directory: {:?}", e);
                }
            }
        } else {
            println!("Error: Filesystem not initialized");
        }
    }

    fn cmd_cat(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: cat <filename>");
            return;
        }

        let path = self.resolve_path(args[0]);

        if let Some(fs) = crate::fs::root_fs() {
            let fs = fs.lock();
            match fs.read_file(&path) {
                Ok(content) => {
                    println!("\n─────────────────────────────────────");
                    println!("File: {}", path);
                    println!("─────────────────────────────────────");
                    
                    // Try to display as UTF-8 text
                    match core::str::from_utf8(&content) {
                        Ok(text) => println!("{}", text),
                        Err(_) => {
                            println!("(Binary file - {} bytes)", content.len());
                            // Show hex dump for binary files
                            for (i, chunk) in content.chunks(16).enumerate() {
                                print!("{:04x}: ", i * 16);
                                for byte in chunk {
                                    print!("{:02x} ", byte);
                                }
                                println!();
                                if i >= 10 {
                                    println!("... ({} more bytes)", content.len() - (i + 1) * 16);
                                    break;
                                }
                            }
                        }
                    }
                    println!("─────────────────────────────────────\n");
                }
                Err(e) => {
                    println!("Error reading file: {:?}", e);
                }
            }
        } else {
            println!("Error: Filesystem not initialized");
        }
    }

    fn cmd_mkdir(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: mkdir <directory>");
            return;
        }

        let path = self.resolve_path(args[0]);

        if let Some(fs) = crate::fs::root_fs() {
            let mut fs = fs.lock();
            match fs.create_dir(&path) {
                Ok(_) => println!("Directory created: {}", path),
                Err(e) => println!("Error creating directory: {:?}", e),
            }
        } else {
            println!("Error: Filesystem not initialized");
        }
    }

    fn cmd_touch(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: touch <filename>");
            return;
        }

        let path = self.resolve_path(args[0]);

        if let Some(fs) = crate::fs::root_fs() {
            let mut fs = fs.lock();
            match fs.create_file(&path, b"") {
                Ok(_) => println!("File created: {}", path),
                Err(e) => println!("Error creating file: {:?}", e),
            }
        } else {
            println!("Error: Filesystem not initialized");
        }
    }

    async fn cmd_run(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: run <script>");
            println!("Available scripts:");
            self.list_scripts();
            return;
        }

        let script_name = args[0];
        let path = if script_name.starts_with('/') {
            script_name.to_string()
        } else if script_name.contains('/') {
            self.resolve_path(script_name)
        } else {
            // Try to find in /scripts directory
            format!("/scripts/{}", script_name)
        };

        // Try to read from filesystem
        if let Some(fs) = crate::fs::root_fs() {
            let fs = fs.lock();
            match fs.read_file(&path) {
                Ok(content) => {
                    match core::str::from_utf8(&content) {
                        Ok(source) => {
                            println!("\n─────────────────────────────────────");
                            println!("Executing: {}", path);
                            println!("─────────────────────────────────────");
                            
                            match rustrial_script::run(source) {
                                Ok(_) => {
                                    println!("\n─────────────────────────────────────");
                                    println!("Script completed successfully");
                                    println!("─────────────────────────────────────\n");
                                }
                                Err(e) => {
                                    println!("\nScript error: {}", e);
                                }
                            }
                        }
                        Err(_) => println!("Error: File is not valid UTF-8 text"),
                    }
                }
                Err(e) => {
                    println!("Error: Could not read file '{}': {:?}", path, e);
                    println!("\nAvailable scripts:");
                    self.list_scripts();
                }
            }
        } else {
            println!("Error: Filesystem not initialized");
        }
    }

    fn list_scripts(&self) {
        if let Some(fs) = crate::fs::root_fs() {
            let fs = fs.lock();
            if let Ok(entries) = fs.read_dir("/scripts") {
                for entry in entries {
                    if entry.file_type == FileType::File {
                        println!("  - {}", entry.name);
                    }
                }
            }
        }
    }

    fn cmd_cd(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.current_dir = String::from("/");
            return;
        }

        let new_path = self.resolve_path(args[0]);

        // Verify directory exists
        if let Some(fs) = crate::fs::root_fs() {
            let fs = fs.lock();
            match fs.read_dir(&new_path) {
                Ok(_) => {
                    self.current_dir = new_path;
                }
                Err(_) => {
                    println!("Error: Directory not found: {}", new_path);
                }
            }
        } else {
            println!("Error: Filesystem not initialized");
        }
    }

    fn cmd_pwd(&self) {
        println!("{}", self.current_dir);
    }

    fn cmd_color(&mut self, args: &[&str]) {
        if args.len() < 2 {
            println!("Usage: color <foreground> <background>");
            println!("Colors: 0-15 (see 'help' for color list)");
            return;
        }

        let fg = match args[0].parse::<u8>() {
            Ok(n) if n <= 15 => n,
            _ => {
                println!("Invalid foreground color. Must be 0-15.");
                return;
            }
        };

        let bg = match args[1].parse::<u8>() {
            Ok(n) if n <= 15 => n,
            _ => {
                println!("Invalid background color. Must be 0-15.");
                return;
            }
        };

        self.foreground_color = Self::color_from_u8(fg);
        self.background_color = Self::color_from_u8(bg);
        
        WRITER.lock().set_color(self.foreground_color, self.background_color);
        println!("Color changed!");
    }

    fn color_from_u8(value: u8) -> Color {
        match value {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::Pink,
            14 => Color::Yellow,
            15 => Color::White,
            _ => Color::LightGray,
        }
    }

    /// Resolve a path relative to current directory
    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            // Absolute path
            path.to_string()
        } else if path == "." {
            self.current_dir.clone()
        } else if path == ".." {
            // Go up one directory
            if self.current_dir == "/" {
                String::from("/")
            } else {
                let mut parts: Vec<&str> = self.current_dir.split('/').collect();
                parts.pop();
                if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
                    String::from("/")
                } else {
                    parts.join("/")
                }
            }
        } else {
            // Relative path
            if self.current_dir == "/" {
                format!("/{}", path)
            } else {
                format!("{}/{}", self.current_dir, path)
            }
        }
    }
}
