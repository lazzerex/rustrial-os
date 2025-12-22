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
use crate::fs::FileSystem;
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
                                            use crate::vga_buffer::backspace;
                                            backspace();
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
                                    KeyCode::Backspace => {
                                        // Handle backspace as RawKey
                                        if !self.input_buffer.is_empty() {
                                            self.input_buffer.pop();
                                            use crate::vga_buffer::backspace;
                                            backspace();
                                        }
                                    }
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
            "rustrialfetch" | "fetch" => self.cmd_rustrialfetch(),
            "netinfo" => self.cmd_netinfo(args),
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
        println!("  rustrialfetch     - Display system information");
        println!("  netinfo [test]    - Display networking status (use 'test' to allocate DMA)");
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
            match fs.list_dir(&full_path) {
                Ok(entries) => {
                    println!("\nDirectory: {}", full_path);
                    println!("─────────────────────────────────────");
                    
                    if entries.is_empty() {
                        println!("  (empty)");
                    } else {
                        for entry_path in entries {
                            let is_dir = fs.is_dir(&entry_path);
                            let type_str = if is_dir { "[DIR] " } else { "[FILE]" };
                            
                            // Extract just the filename from the full path
                            let name = entry_path.rsplit('/').next().unwrap_or(&entry_path);
                            
                            let size_str = if !is_dir {
                                if let Ok(content) = fs.read_file(&entry_path) {
                                    format!(" ({} bytes)", content.len())
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };
                            println!("  {} {}{}", type_str, name, size_str);
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
            if let Ok(entries) = fs.list_dir("/scripts") {
                for entry_path in entries {
                    if fs.is_file(&entry_path) {
                        let name = entry_path.rsplit('/').next().unwrap_or(&entry_path);
                        println!("  - {}", name);
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
            if fs.is_dir(&new_path) {
                self.current_dir = new_path;
            } else {
                println!("Error: Directory not found: {}", new_path);
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

    fn cmd_rustrialfetch(&self) {
        use crate::native_ffi;
        
        // ASCII art logo
        let logo = [
            "    ____             __       ",
            "   / __ \\__  _______/ /_____ _",
            "  / /_/ / / / / ___/ __/ __ `/",
            " / _, _/ /_/ (__  ) /_/ /_/ / ",
            "/_/ |_|\\__,_/____/\\__/\\__,_/  ",
            "                                ",
        ];
        
        // Get system info
        let cpu_info = native_ffi::CpuInfo::get();
        let dt = native_ffi::DateTime::read();
        let pci_devices = native_ffi::enumerate_pci_devices();
        
        // Count scripts
        let script_count = if let Some(fs) = crate::fs::root_fs() {
            fs.lock().list_dir("/scripts")
                .map(|scripts| scripts.len())
                .unwrap_or(0)
        } else {
            0
        };
        
        println!();
        
        // Display info alongside ASCII art
        let info_lines = [
            format!("OS: RustrialOS v0.1"),
            format!("Kernel: Rust bare-metal"),
            format!("CPU: {}", cpu_info.brand_str()),
            format!("Vendor: {}", cpu_info.vendor_str()),
            format!("Date: {}", dt),
            format!("PCI Devices: {}", pci_devices.len()),
            format!("Scripts: {}", script_count),
            format!("Shell: RustrialShell"),
        ];
        
        // Print logo and info side by side
        for i in 0..logo.len().max(info_lines.len()) {
            // Print logo line (left column)
            if i < logo.len() {
                print!("{}", logo[i]);
            } else {
                // Empty space for alignment
                print!("                                ");
            }
            
            // Print info line (right column)
            if i < info_lines.len() {
                println!("  {}", info_lines[i]);
            } else {
                println!();
            }
        }
        
        println!();
    }

    fn cmd_netinfo(&self, args: &[&str]) {
        println!("\n╔════════════════════════════════════════════════════════════════════╗");
        println!("║              Network Stack Status - Phase 1.1                      ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!("║ Heap Size:        2 MB (expanded for networking)                  ║");
        println!("║ DMA Region:       1 MB allocated                                   ║");
        println!("║ Ring Buffers:     256 × 2KB (RX/TX)                                ║");
        println!("║ Status:           Infrastructure ready ✓                           ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!("║ Phase 1.1:        ✓ Enhanced Memory Management                    ║");
        println!("║ Phase 1.2:        ⏳ PCI Driver Enhancement (pending)              ║");
        println!("║ Phase 2:          ⏳ Network Driver (RTL8139/E1000)                ║");
        println!("║ Phase 3:          ⏳ Ethernet/ARP Protocol                         ║");
        println!("║ Phase 4:          ⏳ IP/ICMP Protocol (ping)                       ║");
        println!("╚════════════════════════════════════════════════════════════════════╝\n");

        if args.len() > 0 && args[0] == "test" {
            println!("Testing DMA allocation...");
            
            use crate::memory::dma;
            
            // Test allocation
            match dma::allocate_dma_buffer(1024) {
                Ok(buffer) => {
                    println!("✓ DMA Buffer allocated successfully!");
                    println!("  Virtual Address:  0x{:x}", buffer.virt_addr.as_u64());
                    println!("  Physical Address: 0x{:x}", buffer.phys_addr.as_u64());
                    println!("  Size:             {} bytes (aligned to {})", 
                             buffer.size, buffer.size);
                    
                    // Test multiple allocations
                    match dma::allocate_dma_buffer(2048) {
                        Ok(buffer2) => {
                            println!("✓ Second DMA buffer allocated!");
                            println!("  Virtual Address:  0x{:x}", buffer2.virt_addr.as_u64());
                            println!("  Physical Address: 0x{:x}", buffer2.phys_addr.as_u64());
                        }
                        Err(e) => {
                            println!("✗ Second allocation failed: {:?}", e);
                        }
                    }
                    
                    // Test ring buffer
                    use crate::net::buffer::PacketRingBuffer;
                    let mut ring: PacketRingBuffer<4, 2048> = PacketRingBuffer::new();
                    let test_packet = [0xAA, 0xBB, 0xCC, 0xDD];
                    
                    match ring.push(&test_packet) {
                        Ok(_) => {
                            println!("✓ Ring buffer push successful!");
                            match ring.pop() {
                                Ok((data, len)) => {
                                    println!("✓ Ring buffer pop successful!");
                                    println!("  Packet length: {} bytes", len);
                                    print!("  Data: ");
                                    for i in 0..len {
                                        print!("{:02X} ", data[i]);
                                    }
                                    println!();
                                }
                                Err(e) => println!("✗ Ring buffer pop failed: {:?}", e),
                            }
                        }
                        Err(e) => println!("✗ Ring buffer push failed: {:?}", e),
                    }
                }
                Err(e) => {
                    println!("✗ DMA allocation failed: {:?}", e);
                    println!("  This might indicate DMA was not properly initialized.");
                }
            }
            println!();
        } else {
            println!("Tip: Use 'netinfo test' to run DMA allocation tests\n");
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
