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
const MAX_SCROLLBACK_LINES: usize = 1000;

pub struct Shell {
    input_buffer: String,
    history: Vec<String>,
    history_index: usize,
    current_dir: String,
    foreground_color: Color,
    background_color: Color,
    scrollback_buffer: Vec<String>,
    scroll_offset: usize,
    in_scroll_mode: bool,
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
            scrollback_buffer: Vec::new(),
            scroll_offset: 0,
            in_scroll_mode: false,
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

    fn print_welcome(&mut self) {
        self.add_to_scrollback("\n╔════════════════════════════════════════════════════════════════════╗");
        self.add_to_scrollback("║              Welcome to RustrialOS Shell v0.1                      ║");
        self.add_to_scrollback("╠════════════════════════════════════════════════════════════════════╣");
        self.add_to_scrollback("║  Type 'help' for available commands                               ║");
        self.add_to_scrollback("║  Type 'exit' to return to desktop                                 ║");
        self.add_to_scrollback("║  Use PageUp/PageDown to scroll through output                     ║");
        self.add_to_scrollback("╚════════════════════════════════════════════════════════════════════╝\n");
        println!("\n╔════════════════════════════════════════════════════════════════════╗");
        println!("║              Welcome to RustrialOS Shell v0.1                      ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!("║  Type 'help' for available commands                               ║");
        println!("║  Type 'exit' to return to desktop                                 ║");
        println!("║  Use PageUp/PageDown to scroll through output                     ║");
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
                                // Exit scroll mode when user starts typing
                                if self.in_scroll_mode {
                                    if character != '\n' {
                                        self.exit_scroll_mode();
                                        // Redraw prompt with any existing input
                                        self.print_prompt();
                                        if !self.input_buffer.is_empty() {
                                            print!("{}", self.input_buffer);
                                        }
                                    }
                                }
                                
                                match character {
                                    '\n' => {
                                        println!();
                                        let result = self.input_buffer.clone();
                                        // Add the command line to scrollback only if not empty
                                        if !result.is_empty() {
                                            self.add_to_scrollback(&format!("{}{} {}", 
                                                self.current_dir, PROMPT, result));
                                        }
                                        return Some(result);
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
                                        // Exit scroll mode
                                        if self.in_scroll_mode {
                                            self.exit_scroll_mode();
                                            self.print_prompt();
                                            if !self.input_buffer.is_empty() {
                                                print!("{}", self.input_buffer);
                                            }
                                        }
                                        // Handle backspace as RawKey
                                        if !self.input_buffer.is_empty() {
                                            self.input_buffer.pop();
                                            use crate::vga_buffer::backspace;
                                            backspace();
                                        }
                                    }
                                    KeyCode::ArrowUp => {
                                        // Exit scroll mode first
                                        if self.in_scroll_mode {
                                            self.exit_scroll_mode();
                                            self.print_prompt();
                                        }
                                        if self.history_index > 0 {
                                            self.history_index -= 1;
                                            self.load_history_entry();
                                        }
                                    }
                                    KeyCode::ArrowDown => {
                                        // Exit scroll mode first
                                        if self.in_scroll_mode {
                                            self.exit_scroll_mode();
                                            self.print_prompt();
                                        }
                                        if self.history_index < self.history.len() {
                                            self.history_index += 1;
                                            self.load_history_entry();
                                        }
                                    }
                                    KeyCode::PageUp => {
                                        self.scroll_up();
                                    }
                                    KeyCode::PageDown => {
                                        self.scroll_down();
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

        // Capture the output by intercepting commands
        match command {
            "help" => self.cmd_help(),
            "clear" | "cls" => {
                self.cmd_clear();
                // Clear scrollback when clearing screen
                self.scrollback_buffer.clear();
                self.scroll_offset = 0;
                self.in_scroll_mode = false;
            },
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
            "pciinfo" => self.cmd_pciinfo(args),
            "exit" | "quit" => return true,
            _ => {
                let msg = format!("Unknown command: '{}'. Type 'help' for available commands.", command);
                self.sprintln(&msg);
            }
        }

        false
    }

    /// Shell println - prints to screen and adds to scrollback
    fn sprintln(&mut self, s: &str) {
        self.add_to_scrollback(s);
        println!("{}", s);
    }

    /// Shell print - prints to screen and adds to scrollback (no newline)
    fn sprint(&mut self, s: &str) {
        self.add_to_scrollback(s);
        print!("{}", s);
    }

    fn cmd_help(&mut self) {
        self.sprintln("\nAvailable Commands:");
        self.sprintln("  help              - Show this help message");
        self.sprintln("  clear, cls        - Clear the screen");
        self.sprintln("  echo <text>       - Print text to the screen");
        self.sprintln("  ls [path]         - List files and directories");
        self.sprintln("  cat <file>        - Display file contents");
        self.sprintln("  mkdir <dir>       - Create a directory");
        self.sprintln("  touch <file>      - Create an empty file");
        self.sprintln("  run <script>      - Execute a RustrialScript file");
        self.sprintln("  cd <dir>          - Change current directory");
        self.sprintln("  pwd               - Print working directory");
        self.sprintln("  color <fg> <bg>   - Change text color (0-15)");
        self.sprintln("  rustrialfetch     - Display system information");
        self.sprintln("  netinfo [test]    - Display networking status (use 'test' to allocate DMA)");
        self.sprintln("  pciinfo [detail]  - Display PCI devices (use 'detail' for BAR info)");
        self.sprintln("  exit, quit        - Return to desktop");
        self.sprintln("\nColors: 0=Black, 1=Blue, 2=Green, 3=Cyan, 4=Red, 5=Magenta, 6=Brown,");
        self.sprintln("        7=LightGray, 8=DarkGray, 9=LightBlue, 10=LightGreen, 11=LightCyan,");
        self.sprintln("        12=LightRed, 13=Pink, 14=Yellow, 15=White\n");
    }

    fn cmd_clear(&self) {
        WRITER.lock().clear_screen();
    }

    fn cmd_echo(&mut self, args: &[&str]) {
        self.sprintln(&args.join(" "));
    }

    fn cmd_ls(&mut self, args: &[&str]) {
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
                    self.sprintln(&format!("\nDirectory: {}", full_path));
                    self.sprintln("─────────────────────────────────────");
                    
                    if entries.is_empty() {
                        self.sprintln("  (empty)");
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
                            self.sprintln(&format!("  {} {}{}", type_str, name, size_str));
                        }
                    }
                    self.sprintln("");
                }
                Err(e) => {
                    self.sprintln(&format!("Error listing directory: {:?}", e));
                }
            }
        } else {
            self.sprintln("Error: Filesystem not initialized");
        }
    }

    fn cmd_cat(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.sprintln("Usage: cat <filename>");
            return;
        }

        let path = self.resolve_path(args[0]);

        if let Some(fs) = crate::fs::root_fs() {
            let fs = fs.lock();
            match fs.read_file(&path) {
                Ok(content) => {
                    self.sprintln("\n─────────────────────────────────────");
                    self.sprintln(&format!("File: {}", path));
                    self.sprintln("─────────────────────────────────────");
                    
                    // Try to display as UTF-8 text
                    match core::str::from_utf8(&content) {
                        Ok(text) => self.sprintln(text),
                        Err(_) => {
                            self.sprintln(&format!("(Binary file - {} bytes)", content.len()));
                            // Show hex dump for binary files
                            for (i, chunk) in content.chunks(16).enumerate() {
                                let mut hex_line = format!("{:04x}: ", i * 16);
                                for byte in chunk {
                                    hex_line.push_str(&format!("{:02x} ", byte));
                                }
                                self.sprintln(&hex_line);
                                if i >= 10 {
                                    self.sprintln(&format!("... ({} more bytes)", content.len() - (i + 1) * 16));
                                    break;
                                }
                            }
                        }
                    }
                    self.sprintln("─────────────────────────────────────\n");
                }
                Err(e) => {
                    self.sprintln(&format!("Error reading file: {:?}", e));
                }
            }
        } else {
            self.sprintln("Error: Filesystem not initialized");
        }
    }

    fn cmd_mkdir(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.sprintln("Usage: mkdir <directory>");
            return;
        }

        let path = self.resolve_path(args[0]);

        if let Some(fs) = crate::fs::root_fs() {
            let mut fs = fs.lock();
            match fs.create_dir(&path) {
                Ok(_) => self.sprintln(&format!("Directory created: {}", path)),
                Err(e) => self.sprintln(&format!("Error creating directory: {:?}", e)),
            }
        } else {
            self.sprintln("Error: Filesystem not initialized");
        }
    }

    fn cmd_touch(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.sprintln("Usage: touch <filename>");
            return;
        }

        let path = self.resolve_path(args[0]);

        if let Some(fs) = crate::fs::root_fs() {
            let mut fs = fs.lock();
            match fs.create_file(&path, b"") {
                Ok(_) => self.sprintln(&format!("File created: {}", path)),
                Err(e) => self.sprintln(&format!("Error creating file: {:?}", e)),
            }
        } else {
            self.sprintln("Error: Filesystem not initialized");
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
                self.sprintln(&format!("Error: Directory not found: {}", new_path));
            }
        } else {
            self.sprintln("Error: Filesystem not initialized");
        }
    }

    fn cmd_pwd(&mut self) {
        self.sprintln(&self.current_dir.clone());
    }

    fn cmd_color(&mut self, args: &[&str]) {
        if args.len() < 2 {
            self.sprintln("Usage: color <foreground> <background>");
            self.sprintln("Colors: 0-15 (see 'help' for color list)");
            return;
        }

        let fg = match args[0].parse::<u8>() {
            Ok(n) if n <= 15 => n,
            _ => {
                self.sprintln("Invalid foreground color. Must be 0-15.");
                return;
            }
        };

        let bg = match args[1].parse::<u8>() {
            Ok(n) if n <= 15 => n,
            _ => {
                self.sprintln("Invalid background color. Must be 0-15.");
                return;
            }
        };

        self.foreground_color = Self::color_from_u8(fg);
        self.background_color = Self::color_from_u8(bg);
        
        WRITER.lock().set_color(self.foreground_color, self.background_color);
        self.sprintln("Color changed!");
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
        println!("║ Status:           Infrastructure ready [OK]                        ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!("║ Phase 1.1:        [OK] Enhanced Memory Management                 ║");
        println!("║ Phase 1.2:        [PENDING] PCI Driver Enhancement                ║");
        println!("║ Phase 2:          [PENDING] Network Driver (RTL8139/E1000)        ║");
        println!("║ Phase 3:          [PENDING] Ethernet/ARP Protocol                 ║");
        println!("║ Phase 4:          [PENDING] IP/ICMP Protocol (ping)               ║");
        println!("╚════════════════════════════════════════════════════════════════════╝\n");

        if args.len() > 0 && args[0] == "test" {
            println!("Testing DMA allocation...");
            
            use crate::memory::dma;
            
            // Test allocation
            match dma::allocate_dma_buffer(1024) {
                Ok(buffer) => {
                    println!("[OK] DMA Buffer allocated successfully!");
                    println!("  Virtual Address:  0x{:x}", buffer.virt_addr.as_u64());
                    println!("  Physical Address: 0x{:x}", buffer.phys_addr.as_u64());
                    println!("  Size:             {} bytes (aligned to {})", 
                             buffer.size, buffer.size);
                    
                    // Test multiple allocations
                    match dma::allocate_dma_buffer(2048) {
                        Ok(buffer2) => {
                            println!("[OK] Second DMA buffer allocated!");
                            println!("  Virtual Address:  0x{:x}", buffer2.virt_addr.as_u64());
                            println!("  Physical Address: 0x{:x}", buffer2.phys_addr.as_u64());
                        }
                        Err(e) => {
                            println!("[FAIL] Second allocation failed: {:?}", e);
                        }
                    }
                    
                    // Test ring buffer
                    use crate::net::buffer::PacketRingBuffer;
                    let mut ring: PacketRingBuffer<4, 2048> = PacketRingBuffer::new();
                    let test_packet = [0xAA, 0xBB, 0xCC, 0xDD];
                    
                    match ring.push(&test_packet) {
                        Ok(_) => {
                            println!("[OK] Ring buffer push successful!");
                            match ring.pop() {
                                Ok((data, len)) => {
                                    println!("[OK] Ring buffer pop successful!");
                                    println!("  Packet length: {} bytes", len);
                                    print!("  Data: ");
                                    for i in 0..len {
                                        print!("{:02X} ", data[i]);
                                    }
                                    println!();
                                }
                                Err(e) => println!("[FAIL] Ring buffer pop failed: {:?}", e),
                            }
                        }
                        Err(e) => println!("[FAIL] Ring buffer push failed: {:?}", e),
                    }
                }
                Err(e) => {
                    println!("[FAIL] DMA allocation failed: {:?}", e);
                    println!("  This might indicate DMA was not properly initialized.");
                }
            }
            println!();
        } else {
            println!("Tip: Use 'netinfo test' to run DMA allocation tests\n");
        }
    }

    fn cmd_pciinfo(&self, args: &[&str]) {
        use crate::native_ffi;
        
        let devices = native_ffi::enumerate_pci_devices();
        let show_detail = args.len() > 0 && args[0] == "detail";
        
        println!("\n╔════════════════════════════════════════════════════════════════════╗");
        println!("║              PCI Device Information - Stage 1.2                    ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!("║ Total Devices: {:<53} ║", devices.len());
        println!("╚════════════════════════════════════════════════════════════════════╝\n");
        
        for (idx, device) in devices.iter().enumerate() {
            println!("Device #{}: {}", idx + 1, device);
            
            if show_detail {
                println!("  Bus:Function:Device: {:02X}:{:02X}.{}", 
                         device.bus, device.device, device.function);
                println!("  Vendor:Device ID:    {:04X}:{:04X}", 
                         device.vendor_id, device.device_id);
                println!("  Class:Subclass:      {:02X}:{:02X}", 
                         device.class_code, device.subclass);
                println!("  Interrupt:           Line={} Pin={}", 
                         device.interrupt_line, device.interrupt_pin);
                
                // Display BARs
                println!("  Base Address Registers (BARs):");
                for bar_idx in 0..6 {
                    if let Some(bar) = native_ffi::pci_get_bar(device, bar_idx) {
                        let bar_type = if bar.is_mmio { "MMIO" } else { "I/O " };
                        println!("    BAR{}: {} @ 0x{:016x} (size: {} bytes / {} KB)", 
                                 bar_idx, bar_type, bar.base_addr.as_u64(), 
                                 bar.size, bar.size / 1024);
                    }
                }
                
                // Check if network device
                if device.class_code == 0x02 { // Network controller
                    println!("  [!] NETWORK DEVICE DETECTED - Ready for driver!");
                    if device.vendor_id == 0x10EC && device.device_id == 0x8139 {
                        println!("  [*] RTL8139 Found - Recommended for Stage 2!");
                    } else if device.vendor_id == 0x8086 && device.device_id == 0x100E {
                        println!("  [*] Intel E1000 Found - Alternative for Stage 2!");
                    }
                }
                
                println!();
            }
        }
        
        if !show_detail {
            println!("\nTip: Use 'pciinfo detail' to see BAR mappings and IRQ configuration\n");
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

    /// Add a line to the scrollback buffer
    fn add_to_scrollback(&mut self, line: &str) {
        // Split by newlines and add each line separately
        for l in line.split('\n') {
            if self.scrollback_buffer.len() >= MAX_SCROLLBACK_LINES {
                self.scrollback_buffer.remove(0);
            }
            self.scrollback_buffer.push(l.to_string());
        }
    }

    /// Scroll up in the scrollback buffer
    fn scroll_up(&mut self) {
        use crate::vga_buffer::BUFFER_HEIGHT;
        
        if self.scrollback_buffer.is_empty() {
            return;
        }

        // Enter scroll mode if not already in it
        if !self.in_scroll_mode {
            self.in_scroll_mode = true;
            self.scroll_offset = 0;
        }

        // Scroll up by half a screen
        let scroll_amount = BUFFER_HEIGHT / 2;
        let max_offset = self.scrollback_buffer.len().saturating_sub(BUFFER_HEIGHT);
        
        if self.scroll_offset < max_offset {
            self.scroll_offset = (self.scroll_offset + scroll_amount).min(max_offset);
            self.redraw_screen();
        }
    }

    /// Scroll down in the scrollback buffer
    fn scroll_down(&mut self) {
        use crate::vga_buffer::BUFFER_HEIGHT;
        
        if !self.in_scroll_mode {
            return;
        }
        
        if self.scroll_offset == 0 {
            return;
        }

        // Scroll down by half a screen
        let scroll_amount = BUFFER_HEIGHT / 2;
        
        if self.scroll_offset > scroll_amount {
            self.scroll_offset -= scroll_amount;
        } else {
            self.scroll_offset = 0;
        }
        
        self.redraw_screen();
    }

    /// Redraw the screen from the scrollback buffer
    fn redraw_screen(&self) {
        use crate::vga_buffer::{BUFFER_HEIGHT, WRITER};
        
        // Clear screen
        WRITER.lock().clear_screen();
        
        // Calculate which lines to display
        let total_lines = self.scrollback_buffer.len();
        let display_lines = BUFFER_HEIGHT - 1; // Reserve last line for prompt/status
        
        if total_lines <= display_lines {
            // All lines fit on screen
            for line in &self.scrollback_buffer {
                println!("{}", line);
            }
        } else {
            // Display from offset
            let start_idx = total_lines.saturating_sub(display_lines + self.scroll_offset);
            let end_idx = (start_idx + display_lines).min(total_lines);
            
            for i in start_idx..end_idx {
                println!("{}", self.scrollback_buffer[i]);
            }
        }
        
        // Show scroll indicator if in scroll mode
        if self.in_scroll_mode {
            let scroll_pct = if total_lines > display_lines {
                100 - (self.scroll_offset * 100 / (total_lines - display_lines))
            } else {
                100
            };
            print!("[SCROLL MODE: {}% - PgUp/PgDn to scroll, any key to exit]", scroll_pct);
        }
    }

    /// Exit scroll mode and return to normal operation
    fn exit_scroll_mode(&mut self) {
        if self.in_scroll_mode {
            self.in_scroll_mode = false;
            self.scroll_offset = 0;
            // Clear screen and show recent history
            use crate::vga_buffer::WRITER;
            WRITER.lock().clear_screen();
            
            // Show the last screen's worth of scrollback
            use crate::vga_buffer::BUFFER_HEIGHT;
            let display_lines = BUFFER_HEIGHT.saturating_sub(2); // Leave room for prompt
            let start = self.scrollback_buffer.len().saturating_sub(display_lines);
            
            for i in start..self.scrollback_buffer.len() {
                println!("{}", self.scrollback_buffer[i]);
            }
        }
    }
}
