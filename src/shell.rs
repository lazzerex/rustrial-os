//! Simple shell/command interpreter for rustrial_os

use crate::fs::vfs::{self, VfsNode};
use crate::rustrial_script::vm;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::vga_buffer::println;

pub struct Shell {
    pub prompt: &'static str,
}

impl Shell {
    pub fn new() -> Self {
        Shell { prompt: "rustrial> " }
    }

    pub fn run(&self) {
        println!("Welcome to the rustrial_os shell! Type 'help' for commands.");
        loop {
            print!("{}", self.prompt);
            let input = self.read_line();
            let args = Shell::parse_command(&input);
            if args.is_empty() { continue; }
            match args[0].as_str() {
                "ls" => self.cmd_ls(args.get(1)),
                "cat" => self.cmd_cat(args.get(1)),
                "run" => self.cmd_run(args.get(1)),
                "help" => self.cmd_help(),
                "clear" => self.cmd_clear(),
                "echo" => self.cmd_echo(&args[1..]),
                "exit" => break,
                _ => println!("Unknown command: {}", args[0]),
            }
        }
    }

    fn read_line(&self) -> String {
        // TODO: Replace with real input method (keyboard buffer)
        // For now, stub for integration
        String::new()
    }

    fn parse_command(input: &str) -> Vec<String> {
        input.split_whitespace().map(|s| s.to_string()).collect()
    }

    fn cmd_ls(&self, path: Option<&String>) {
        let dir = path.map(|s| s.as_str()).unwrap_or("/");
        match vfs::list_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    println!("{}", entry);
                }
            }
            Err(e) => println!("ls error: {}", e),
        }
    }

    fn cmd_cat(&self, path: Option<&String>) {
        if let Some(path) = path {
            match vfs::read_file(path) {
                Ok(content) => println!("{}", content),
                Err(e) => println!("cat error: {}", e),
            }
        } else {
            println!("Usage: cat <file>");
        }
    }

    fn cmd_run(&self, script: Option<&String>) {
        if let Some(script) = script {
            match vm::run_script(script) {
                Ok(_) => println!("Script executed."),
                Err(e) => println!("run error: {}", e),
            }
        } else {
            println!("Usage: run <script>");
        }
    }

    fn cmd_help(&self) {
        println!("Available commands:");
        println!("  ls [dir]      - List files");
        println!("  cat <file>    - Show file contents");
        println!("  run <script>  - Run rustrialscript");
        println!("  echo <text>   - Print text");
        println!("  clear         - Clear screen");
        println!("  help          - Show this help");
        println!("  exit          - Exit shell");
    }

    fn cmd_clear(&self) {
        crate::vga_buffer::clear_screen();
    }

    fn cmd_echo(&self, args: &[String]) {
        println!("{}", args.join(" "));
    }
}
