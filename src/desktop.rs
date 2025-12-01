/// Desktop GUI Environment for Rustrial OS
/// 
/// This module provides a graphical desktop environment with icons,
/// mouse cursor, and window management capabilities.

use alloc::{vec::Vec, string::String};
use crate::vga_buffer::{Color, BUFFER_HEIGHT, BUFFER_WIDTH};
use crate::task::keyboard;
use crate::task::mouse::{MouseStream, get_position, is_left_button_pressed, update_position, update_buttons};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use futures_util::stream::StreamExt;

/// Desktop icon structure
#[derive(Clone)]
pub struct DesktopIcon {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
    pub label: String,
    pub action: IconAction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IconAction {
    OpenMenu,
    SystemInfo,
    Scripts,
    Hardware,
}

impl DesktopIcon {
    pub fn new(x: i16, y: i16, label: &str, action: IconAction) -> Self {
        DesktopIcon {
            x,
            y,
            width: 12,
            height: 5,
            label: String::from(label),
            action,
        }
    }

    pub fn contains_point(&self, x: i16, y: i16) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }

    pub fn render(&self, selected: bool) {
        use crate::graphics::text_graphics::*;
        
        let bg_color = if selected { Color::LightBlue } else { Color::Blue };
        let fg_color = Color::White;
        
        // Draw icon background
        draw_filled_box(
            self.x as usize,
            self.y as usize,
            self.width as usize,
            self.height as usize,
            fg_color,
            bg_color,
        );
        
        // Draw icon border
        draw_box(
            self.x as usize,
            self.y as usize,
            self.width as usize,
            self.height as usize,
            Color::Yellow,
            bg_color,
        );
        
        // Draw icon symbol in the center
        let symbol = match self.action {
            IconAction::OpenMenu => "[*]",
            IconAction::SystemInfo => "[i]",
            IconAction::Scripts => "[S]",
            IconAction::Hardware => "[H]",
        };
        
        write_at(
            self.x as usize + (self.width as usize / 2) - 1,
            self.y as usize + 1,
            symbol,
            Color::Yellow,
            bg_color,
        );
        
        // Draw label below icon
        let label_len = self.label.len();
        let label_x = if label_len < self.width as usize {
            self.x as usize + (self.width as usize - label_len) / 2
        } else {
            self.x as usize
        };
        
        write_at(
            label_x,
            (self.y + 3) as usize,
            &self.label[..core::cmp::min(label_len, self.width as usize)],
            Color::White,
            Color::Cyan,
        );
    }
}

/// Main desktop environment
pub struct Desktop {
    icons: Vec<DesktopIcon>,
    selected_icon: Option<usize>,
    last_mouse_x: i16,
    last_mouse_y: i16,
    mouse_visible: bool,
}

impl Desktop {
    pub fn new() -> Self {
        let mut icons = Vec::new();
        
        // Create desktop icons arranged in a grid
        icons.push(DesktopIcon::new(5, 4, "Main Menu", IconAction::OpenMenu));
        icons.push(DesktopIcon::new(20, 4, "System", IconAction::SystemInfo));
        icons.push(DesktopIcon::new(35, 4, "Scripts", IconAction::Scripts));
        icons.push(DesktopIcon::new(50, 4, "Hardware", IconAction::Hardware));
        
        Desktop {
            icons,
            selected_icon: None,
            last_mouse_x: 40,
            last_mouse_y: 12,
            mouse_visible: true,
        }
    }

    fn render_desktop(&self) {
        use crate::vga_buffer::clear_screen;
        use crate::graphics::text_graphics::*;
        
        clear_screen();
        
        // Draw desktop background with gradient effect
        for y in 0..BUFFER_HEIGHT {
            for x in 0..BUFFER_WIDTH {
                write_at(x, y, " ", Color::Black, Color::Cyan);
            }
        }
        
        // Draw title bar
        draw_filled_box(0, 0, BUFFER_WIDTH, 1, Color::White, Color::Blue);
        write_at(2, 0, "RUSTRIAL OS - Desktop Environment", Color::Yellow, Color::Blue);
        
        // Draw system info in corner
        use crate::native_ffi;
        let dt = native_ffi::DateTime::read();
        let time_str = alloc::format!("{:02}:{:02}", dt.hour, dt.minute);
        write_at(BUFFER_WIDTH - time_str.len() - 2, 0, &time_str, Color::White, Color::Blue);
        
        // Draw status bar at bottom
        draw_filled_box(0, BUFFER_HEIGHT - 1, BUFFER_WIDTH, 1, Color::White, Color::DarkGray);
        let status_msg = alloc::format!("Mouse: ({:2},{:2}) | Double-click icons | ESC to exit", self.last_mouse_x, self.last_mouse_y);
        write_at(2, BUFFER_HEIGHT - 1, &status_msg, Color::White, Color::DarkGray);
        
        // Render all icons
        for (idx, icon) in self.icons.iter().enumerate() {
            icon.render(Some(idx) == self.selected_icon);
        }
        
        // Draw mouse cursor
        if self.mouse_visible {
            self.render_cursor(self.last_mouse_x, self.last_mouse_y);
        }
    }

    fn render_cursor(&self, x: i16, y: i16) {
        use crate::vga_buffer::{write_char_at, Color};
        
        if x >= 0 && x < BUFFER_WIDTH as i16 && y >= 0 && y < BUFFER_HEIGHT as i16 {
            // Draw a bright, visible cursor
            write_char_at(x as usize, y as usize, b'X', Color::Black, Color::White);
        }
    }

    fn update_mouse_selection(&mut self, x: i16, y: i16) {
        // Check if mouse is over any icon
        let mut found_icon = None;
        for (idx, icon) in self.icons.iter().enumerate() {
            if icon.contains_point(x, y) {
                found_icon = Some(idx);
                break;
            }
        }
        
        if self.selected_icon != found_icon {
            self.selected_icon = found_icon;
            // Only re-render icons when selection changes
            self.render_icons();
        }
    }
    
    fn render_icons(&self) {
        // Re-render all icons (more efficient than full screen)
        for (idx, icon) in self.icons.iter().enumerate() {
            icon.render(Some(idx) == self.selected_icon);
        }
    }
    
    fn update_cursor_position(&mut self, x: i16, y: i16) {
        // Erase old cursor by redrawing that cell from background
        self.redraw_cell(self.last_mouse_x, self.last_mouse_y);
        
        // Update position
        self.last_mouse_x = x;
        self.last_mouse_y = y;
        
        // Draw new cursor
        self.render_cursor(x, y);
        
        // Update status bar
        self.update_status_bar();
    }
    
    fn redraw_cell(&self, x: i16, y: i16) {
        use crate::vga_buffer::{write_char_at, Color};
        
        if x >= 0 && x < BUFFER_WIDTH as i16 && y >= 0 && y < BUFFER_HEIGHT as i16 {
            let ux = x as usize;
            let uy = y as usize;
            
            // Title bar - blue background
            if uy == 0 {
                write_char_at(ux, uy, b' ', Color::White, Color::Blue);
                return;
            }
            
            // Status bar - dark gray background
            if uy == BUFFER_HEIGHT - 1 {
                write_char_at(ux, uy, b' ', Color::White, Color::DarkGray);
                return;
            }
            
            // Check if it's on an icon - need to fully redraw the icon
            for (idx, icon) in self.icons.iter().enumerate() {
                if icon.contains_point(x, y) {
                    // Redraw entire icon to restore it properly
                    icon.render(Some(idx) == self.selected_icon);
                    return;
                }
            }
            
            // Desktop background - cyan
            write_char_at(ux, uy, b' ', Color::Black, Color::Cyan);
        }
    }
    
    fn update_status_bar(&self) {
        use crate::graphics::text_graphics::{write_at, draw_filled_box};
        
        // Redraw only the status bar
        draw_filled_box(0, BUFFER_HEIGHT - 1, BUFFER_WIDTH, 1, Color::White, Color::DarkGray);
        let status_msg = alloc::format!("Mouse: ({:2},{:2}) | Double-click icons | ESC to exit", self.last_mouse_x, self.last_mouse_y);
        write_at(2, BUFFER_HEIGHT - 1, &status_msg, Color::White, Color::DarkGray);
    }

    fn handle_icon_click(&mut self, icon_idx: usize) -> Option<IconAction> {
        if icon_idx < self.icons.len() {
            Some(self.icons[icon_idx].action)
        } else {
            None
        }
    }

    pub async fn run(&mut self) -> IconAction {
        // Initialize mouse hardware
        crate::task::mouse::init_hardware();
        
        // Render initial desktop
        self.render_desktop();
        
        let mut scancodes = keyboard::ScancodeStream::new();
        let mut kb = Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        );
        
        let mut mouse_stream = MouseStream::new();
        let mut left_button_was_pressed = false;
        let mut double_click_timer = 0u32;
        let mut last_clicked_icon: Option<usize> = None;
        
        loop {
            // Process mouse input
            if let Some(packet) = mouse_stream.try_next() {
                update_position(packet.x_movement, packet.y_movement);
                update_buttons(packet.buttons);
                
                let (mx, my) = get_position();
                
                // Update cursor position
                if mx != self.last_mouse_x || my != self.last_mouse_y {
                    self.last_mouse_x = mx;
                    self.last_mouse_y = my;
                    self.update_mouse_selection(mx, my);
                }
                
                // Handle left button clicks
                let left_pressed = is_left_button_pressed();
                if left_pressed && !left_button_was_pressed {
                    // Button just pressed
                    if let Some(icon_idx) = self.selected_icon {
                        // Check for double-click
                        if double_click_timer > 0 && last_clicked_icon == Some(icon_idx) {
                            // Double-click detected!
                            if let Some(action) = self.handle_icon_click(icon_idx) {
                                return action;
                            }
                        } else {
                            // First click
                            last_clicked_icon = Some(icon_idx);
                            double_click_timer = 50; // About 1 second at typical polling rate
                        }
                    }
                }
                left_button_was_pressed = left_pressed;
            }
            
            // Decrement double-click timer
            if double_click_timer > 0 {
                double_click_timer -= 1;
                if double_click_timer == 0 {
                    last_clicked_icon = None;
                }
            }
            
            // Process keyboard input (for keyboard navigation AND cursor control)
            if let Some(scancode) = scancodes.next().await {
                if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                    if let Some(key) = kb.process_keyevent(key_event) {
                        let mut cursor_moved = false;
                        
                        match key {
                            DecodedKey::Unicode('\n') | DecodedKey::Unicode(' ') => {
                                // Enter/Space activates selected icon OR simulates click
                                if let Some(icon_idx) = self.selected_icon {
                                    // Check for double-press
                                    if double_click_timer > 0 && last_clicked_icon == Some(icon_idx) {
                                        if let Some(action) = self.handle_icon_click(icon_idx) {
                                            return action;
                                        }
                                    } else {
                                        last_clicked_icon = Some(icon_idx);
                                        double_click_timer = 50;
                                    }
                                }
                            }
                            // Arrow keys move the cursor
                            DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                                if self.last_mouse_x > 0 {
                                    self.last_mouse_x -= 1;
                                    cursor_moved = true;
                                }
                            }
                            DecodedKey::RawKey(KeyCode::ArrowRight) => {
                                if self.last_mouse_x < BUFFER_WIDTH as i16 - 1 {
                                    self.last_mouse_x += 1;
                                    cursor_moved = true;
                                }
                            }
                            DecodedKey::RawKey(KeyCode::ArrowUp) => {
                                if self.last_mouse_y > 0 {
                                    self.last_mouse_y -= 1;
                                    cursor_moved = true;
                                }
                            }
                            DecodedKey::RawKey(KeyCode::ArrowDown) => {
                                if self.last_mouse_y < BUFFER_HEIGHT as i16 - 1 {
                                    self.last_mouse_y += 1;
                                    cursor_moved = true;
                                }
                            }
                            _ => {}
                        }
                        
                        // Update cursor efficiently if moved
                        if cursor_moved {
                            self.update_cursor_position(self.last_mouse_x, self.last_mouse_y);
                            self.update_mouse_selection(self.last_mouse_x, self.last_mouse_y);
                        }
                    }
                }
            }
            
            // Small delay to prevent busy-waiting
            for _ in 0..1000 {
                core::hint::spin_loop();
            }
        }
    }
}

/// Main desktop loop
pub async fn run_desktop_environment() -> IconAction {
    let mut desktop = Desktop::new();
    desktop.run().await
}
