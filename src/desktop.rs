/// Desktop GUI Environment for Rustrial OS
/// 
/// This module provides a graphical desktop environment with icons,
/// mouse cursor, and window management capabilities.

use alloc::{vec::Vec, string::String};
use crate::vga_buffer::{Color, BUFFER_HEIGHT, BUFFER_WIDTH};
use crate::window_manager::WindowManager;
use crate::context_menu::{ContextMenu, MenuItem, MenuActionKind};
use crate::task::keyboard;
use crate::task::mouse::{MouseStream, get_position, is_left_button_pressed, is_right_button_pressed, update_position, update_buttons};
use crate::rustrial_menu::menu_system::shutdown::{ShutdownButton, shutdown_system};
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
    Shell,
    Shutdown,
    FileManager,
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
        
        // Draw icon background
            // Always clear icon background with bg_color only
            draw_filled_box(
                self.x as usize,
                self.y as usize,
                self.width as usize,
                self.height as usize,
                bg_color,
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
            IconAction::Shell => "[>_]",
            IconAction::Shutdown => "[OFF]",
            IconAction::FileManager => "[F]",
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
    window_manager: WindowManager,
    cursor_under: u16,
    context_menu: Option<ContextMenu>,
}

impl Desktop {
    pub fn new() -> Self {
        let mut icons = Vec::new();
        
        // Create desktop icons arranged in a grid
        icons.push(DesktopIcon::new(5, 4, "Main Menu", IconAction::OpenMenu));
        icons.push(DesktopIcon::new(20, 4, "System", IconAction::SystemInfo));
        icons.push(DesktopIcon::new(35, 4, "Scripts", IconAction::Scripts));
        icons.push(DesktopIcon::new(50, 4, "Hardware", IconAction::Hardware));
        icons.push(DesktopIcon::new(65, 4, "Shell", IconAction::Shell));
        icons.push(DesktopIcon::new(5, 10, "Files", IconAction::FileManager));
        // Shutdown button in bottom-right corner
        icons.push(DesktopIcon {
            x: 64,
            y: 18,
            width: 14,
            height: 5,
            label: String::from("Shut Down"),
            action: IconAction::Shutdown,
        });
        
        Desktop {
            icons,
            selected_icon: None,
            last_mouse_x: 40,
            last_mouse_y: 12,
            mouse_visible: true,
            window_manager: WindowManager::new(),
            cursor_under: 0,
            context_menu: None,
        }
    }

    fn render_desktop(&self) {
        use crate::graphics::text_graphics::*;

        // Draw desktop background with gradient effect
        for y in 0..BUFFER_HEIGHT {
            for x in 0..BUFFER_WIDTH {
                write_at(x, y, " ", Color::Black, Color::Cyan);
            }
        }
        
        // Draw title bar
        draw_filled_box(0, 0, BUFFER_WIDTH, 1, Color::White, Color::Blue);
        write_at(2, 0, "RUSTRIAL OS", Color::Yellow, Color::Blue);

        // Draw local-adjusted date and time in corner (UTC+7)
        use crate::native_ffi;
        let mut dt = native_ffi::DateTime::read();
        let mut hour = dt.hour as i16 + 7;
        let mut day = dt.day as i16;
        let mut month = dt.month as i16;
        let mut year = dt.year as i16;
        let mut weekday = dt.weekday as i16;
        let days_in_month = |month: i16, year: i16| -> i16 {
            match month {
                1 => 31,
                2 => if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) { 29 } else { 28 },
                3 => 31,
                4 => 30,
                5 => 31,
                6 => 30,
                7 => 31,
                8 => 31,
                9 => 30,
                10 => 31,
                11 => 30,
                12 => 31,
                _ => 31,
            }
        };
        if hour >= 24 {
            hour -= 24;
            day += 1;
            weekday = (weekday + 1) % 7;
            if day > days_in_month(month, year) {
                day = 1;
                month += 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
            }
        }
        dt.hour = hour as u8;
        dt.day = day as u8;
        dt.month = month as u8;
        dt.year = year as u16;
        dt.weekday = weekday as u8;
        let time_str = alloc::format!("{:02}:{:02} {} {:02}, {:04}", dt.hour, dt.minute, dt.month_str(), dt.day, dt.year);
        write_at(BUFFER_WIDTH - time_str.len() - 2, 0, &time_str, Color::White, Color::Blue);
        
        self.render_taskbar();
        
        // Render all icons
        for (idx, icon) in self.icons.iter().enumerate() {
            if icon.action == IconAction::Shutdown {
                let shutdown_btn = ShutdownButton {
                    x: icon.x as usize,
                    y: icon.y as usize,
                    width: icon.width as usize,
                    height: icon.height as usize,
                };
                shutdown_btn.render(true);
            } else {
                icon.render(Some(idx) == self.selected_icon);
            }
        }
    }

    fn render_cursor(&mut self, x: i16, y: i16) {
        if x >= 0 && x < BUFFER_WIDTH as i16 && y >= 0 && y < BUFFER_HEIGHT as i16 {
            use x86_64::instructions::interrupts;
            self.cursor_under = interrupts::without_interrupts(|| unsafe {
                core::ptr::read_volatile(
                    (0xb8000 as *const u16).add(y as usize * BUFFER_WIDTH + x as usize)
                )
            });
            use crate::vga_buffer::{write_char_at, Color};
            write_char_at(x as usize, y as usize, b'^', Color::Black, Color::White);
        }
    }

    fn restore_cursor_under(&self, x: i16, y: i16) {
        if x >= 0 && x < BUFFER_WIDTH as i16 && y >= 0 && y < BUFFER_HEIGHT as i16 {
            use x86_64::instructions::interrupts;
            let val = self.cursor_under;
            interrupts::without_interrupts(|| unsafe {
                core::ptr::write_volatile(
                    (0xb8000 as *mut u16).add(y as usize * BUFFER_WIDTH + x as usize),
                    val,
                );
            });
        }
    }

    fn render_icons(&self) {
        for (idx, icon) in self.icons.iter().enumerate() {
            icon.render(Some(idx) == self.selected_icon);
        }
    }

    fn update_mouse_selection(&mut self, x: i16, y: i16) -> bool {
        let mut found_icon = None;
        for (idx, icon) in self.icons.iter().enumerate() {
            if icon.contains_point(x, y) {
                found_icon = Some(idx);
                break;
            }
        }
        if self.selected_icon != found_icon {
            self.selected_icon = found_icon;
            true
        } else {
            false
        }
    }

    fn update_cursor_position(&mut self, x: i16, y: i16) -> bool {
        self.restore_cursor_under(self.last_mouse_x, self.last_mouse_y);
        self.last_mouse_x = x;
        self.last_mouse_y = y;
        let need_redraw = if self.context_menu.is_some() {
            self.update_context_menu_hover(x, y);
            false
        } else if self.window_manager.is_point_over_window(x, y) {
            if self.selected_icon.is_some() {
                self.selected_icon = None;
                true
            } else {
                false
            }
        } else {
            self.update_mouse_selection(x, y)
        };
        self.render_cursor(x, y);
        need_redraw
    }

    fn erase_desktop_region(&self, x: usize, y: usize, w: usize, h: usize) {
        use crate::graphics::text_graphics::draw_filled_box;
        let x_end = (x + w).min(BUFFER_WIDTH);
        let y_end = (y + h).min(BUFFER_HEIGHT - 1);
        let y = y.max(1);
        if x_end <= x || y_end <= y { return; }
        draw_filled_box(x, y, x_end - x, y_end - y, Color::Black, Color::Cyan);

        for (idx, icon) in self.icons.iter().enumerate() {
            let ix = icon.x as usize;
            let iy = icon.y as usize;
            let iw = icon.width as usize;
            let ih = icon.height as usize;
            if ix < x_end && ix + iw > x && iy < y_end && iy + ih > y {
                if icon.action == IconAction::Shutdown {
                    use crate::rustrial_menu::menu_system::shutdown::ShutdownButton;
                    let btn = ShutdownButton { x: ix, y: iy, width: iw, height: ih };
                    btn.render(true);
                } else {
                    icon.render(Some(idx) == self.selected_icon);
                }
            }
        }
    }
    
    fn render_taskbar(&self) {
        use crate::graphics::text_graphics::{draw_filled_box, write_at};

        draw_filled_box(0, BUFFER_HEIGHT - 1, BUFFER_WIDTH, 1, Color::LightGray, Color::DarkGray);

        // New-window button
        write_at(0, BUFFER_HEIGHT - 1, "[W]", Color::LightGreen, Color::DarkGray);

        // One button per open window: "[title     ]" = 12 chars, +1 gap = 13 stride
        let mut x = 4usize;
        for win in self.window_manager.get_windows() {
            if x + 12 > BUFFER_WIDTH { break; }
            let focused = self.window_manager.is_focused(win.id);
            let btn_bg = if focused { Color::Blue } else { Color::Black };
            let title = if win.title.len() > 10 { &win.title[..10] } else { &win.title };
            let btn = alloc::format!("[{:<10}]", title);
            write_at(x, BUFFER_HEIGHT - 1, &btn, Color::White, btn_bg);
            x += 13;
        }
    }

    fn handle_taskbar_click(&mut self, mx: i16) {
        let mx_u = mx as usize;

        // [W] button (x 0..=2)
        if mx_u <= 2 {
            self.window_manager.add_window(18, 5, 44, 14, "Window");
            return;
        }

        // Collect button positions first to avoid borrow conflict
        let mut buttons: Vec<(u8, usize)> = Vec::new();
        let mut x = 4usize;
        for win in self.window_manager.get_windows() {
            if x + 12 > BUFFER_WIDTH { break; }
            buttons.push((win.id, x));
            x += 13;
        }
        for (id, x_start) in buttons {
            if mx_u >= x_start && mx_u < x_start + 12 {
                self.window_manager.focus_and_raise(id);
                return;
            }
        }
    }

    fn render_context_menu(&self) {
        if let Some(ref menu) = self.context_menu {
            menu.render();
        }
    }

    fn update_context_menu_hover(&mut self, x: i16, y: i16) {
        if let Some(ref mut menu) = self.context_menu {
            if menu.update_hover(x, y) {
                menu.render();
            }
        }
    }

    fn execute_menu_action(&mut self, action: MenuActionKind) {
        match action {
            MenuActionKind::NewWindow => {
                self.window_manager.add_window(18, 5, 44, 14, "Window");
            }
            MenuActionKind::CloseWindow(id) => {
                self.window_manager.close_window(id);
            }
            MenuActionKind::Refresh => {}
        }
    }

    fn handle_icon_click(&mut self, icon_idx: usize) -> Option<IconAction> {
        if icon_idx < self.icons.len() {
            Some(self.icons[icon_idx].action)
        } else {
            None
        }
    }

    pub async fn run(&mut self) -> IconAction {
        // Mouse hardware is already initialized in main.rs
        
        // Render initial desktop
        self.render_desktop();
        self.window_manager.render_all();
        self.render_cursor(self.last_mouse_x, self.last_mouse_y);
        
        // Initialize keyboard - use ScancodeStream to ensure queue is initialized
        let _scancodes = keyboard::ScancodeStream::new();
        let mut kb = Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        );
        
        let mut mouse_stream = MouseStream::new();
        let mut left_button_was_pressed = true;
        let mut right_button_was_pressed = is_right_button_pressed();
        let mut double_click_timer = 0u32;
        let mut last_clicked_icon: Option<usize> = None;
        
        // Drain any stale mouse packets from before entering desktop
        while mouse_stream.try_next().is_some() {}
        
        // Wait for button to be released before accepting clicks
        left_button_was_pressed = is_left_button_pressed();
        
        loop {
            // Capture drag/resize window bounds before processing packets (used for targeted erase)
            let drag_pre_bounds = self.window_manager.current_drag_bounds();

            // Process ALL pending mouse packets (non-blocking)
            let mut need_full_redraw = false;

            while let Some(packet) = mouse_stream.try_next() {
                update_position(packet.x_movement, packet.y_movement);
                update_buttons(packet.buttons);

                let (mx, my) = get_position();
                let left_pressed = packet.left_button();

                if !left_pressed && left_button_was_pressed {
                    self.window_manager.on_mouse_up();
                }

                if mx != self.last_mouse_x || my != self.last_mouse_y {
                    if self.window_manager.on_mouse_move(mx, my) {
                        self.last_mouse_x = mx;
                        self.last_mouse_y = my;
                        need_full_redraw = true;
                    } else {
                        if self.update_cursor_position(mx, my) {
                            need_full_redraw = true;
                        }
                    }
                }

                if left_pressed && !left_button_was_pressed {
                    if let Some(menu) = self.context_menu.take() {
                        if let Some(action) = menu.action_at(mx, my) {
                            self.execute_menu_action(action);
                        }
                        need_full_redraw = true;
                    } else if self.window_manager.on_mouse_down(mx, my) {
                        need_full_redraw = true;
                    } else if my as usize == BUFFER_HEIGHT - 1 {
                        self.handle_taskbar_click(mx);
                        need_full_redraw = true;
                    } else if let Some(icon_idx) = self.selected_icon {
                        if double_click_timer > 0 && last_clicked_icon == Some(icon_idx) {
                            if let Some(action) = self.handle_icon_click(icon_idx) {
                                match action {
                                    IconAction::Shutdown => shutdown_system(),
                                    IconAction::FileManager => {
                                        self.window_manager.add_file_manager(8, 3, 40, 18, "/");
                                        need_full_redraw = true;
                                    }
                                    _ => return action,
                                }
                            }
                        } else {
                            last_clicked_icon = Some(icon_idx);
                            double_click_timer = 5000;
                        }
                    } else {
                        last_clicked_icon = None;
                        double_click_timer = 0;
                    }
                }

                let right_pressed = packet.right_button();
                if right_pressed && !right_button_was_pressed {
                    self.context_menu = None;
                    let mut items: Vec<MenuItem> = Vec::new();
                    if self.window_manager.is_point_over_window(mx, my) {
                        if let Some(id) = self.window_manager.topmost_window_at(mx, my) {
                            items.push(MenuItem::new("Close Window", MenuActionKind::CloseWindow(id)));
                        }
                    } else if my as usize != BUFFER_HEIGHT - 1 {
                        items.push(MenuItem::new("New Window", MenuActionKind::NewWindow));
                        items.push(MenuItem::new("Refresh", MenuActionKind::Refresh));
                    }
                    if !items.is_empty() {
                        self.context_menu = Some(ContextMenu::new(mx as usize, my as usize, items));
                        need_full_redraw = true;
                    }
                }
                right_button_was_pressed = right_pressed;

                left_button_was_pressed = left_pressed;
            }

            if need_full_redraw {
                let is_drag = self.window_manager.is_dragging_or_resizing();
                if is_drag && self.context_menu.is_none() {
                    // Targeted erase: fill union of old+new window bounds with desktop bg,
                    // then redraw windows. Avoids clearing the whole screen on every move.
                    let drag_post_bounds = self.window_manager.current_drag_bounds();
                    if let Some((px, py, pw, ph)) = drag_pre_bounds {
                        let (ex, ey, ew, eh) = if let Some((nx, ny, nw, nh)) = drag_post_bounds {
                            let ex = px.min(nx);
                            let ey = py.min(ny);
                            let ex2 = (px + pw).max(nx + nw);
                            let ey2 = (py + ph).max(ny + nh);
                            (ex, ey, ex2 - ex, ey2 - ey)
                        } else {
                            (px, py, pw, ph)
                        };
                        self.erase_desktop_region(ex, ey, ew, eh);
                    }
                    self.window_manager.render_all();
                    self.render_cursor(self.last_mouse_x, self.last_mouse_y);
                } else {
                    self.render_desktop();
                    self.window_manager.render_all();
                    self.render_context_menu();
                    self.render_cursor(self.last_mouse_x, self.last_mouse_y);
                }
            }
            
            // Decrement double-click timer (slowly)
            if double_click_timer > 0 {
                double_click_timer = double_click_timer.saturating_sub(1);
            }
            
            // Process keyboard input (non-blocking polling)
            while let Some(scancode) = keyboard::try_pop_scancode() {
                if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                    if let Some(key) = kb.process_keyevent(key_event) {
                        match key {
                            DecodedKey::Unicode('\n') | DecodedKey::Unicode(' ') => {
                                // Enter/Space activates selected icon OR simulates click
                                if let Some(icon_idx) = self.selected_icon {
                                    // Check for double-press
                                    if double_click_timer > 0 && last_clicked_icon == Some(icon_idx) {
                                        if let Some(action) = self.handle_icon_click(icon_idx) {
                                            match action {
                                                IconAction::Shutdown => shutdown_system(),
                                                IconAction::FileManager => {
                                                    self.window_manager.add_file_manager(8, 3, 40, 18, "/");
                                                    self.render_desktop();
                                                    self.window_manager.render_all();
                                                    self.render_cursor(self.last_mouse_x, self.last_mouse_y);
                                                }
                                                _ => return action,
                                            }
                                        }
                                    } else {
                                        last_clicked_icon = Some(icon_idx);
                                        double_click_timer = 50;
                                    }
                                }
                            }
                            // Arrow keys move the cursor (update the atomic position like real mouse)
                            DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                                update_position(-2, 0);  // Move left
                                let (mx, my) = get_position();
                                self.update_cursor_position(mx, my);
                                self.update_mouse_selection(mx, my);
                            }
                            DecodedKey::RawKey(KeyCode::ArrowRight) => {
                                update_position(2, 0);  // Move right
                                let (mx, my) = get_position();
                                self.update_cursor_position(mx, my);
                                self.update_mouse_selection(mx, my);
                            }
                            DecodedKey::RawKey(KeyCode::ArrowUp) => {
                                update_position(0, 2);  // Move up (inverted Y)
                                let (mx, my) = get_position();
                                self.update_cursor_position(mx, my);
                                self.update_mouse_selection(mx, my);
                            }
                            DecodedKey::RawKey(KeyCode::ArrowDown) => {
                                update_position(0, -2);  // Move down (inverted Y)
                                let (mx, my) = get_position();
                                self.update_cursor_position(mx, my);
                                self.update_mouse_selection(mx, my);
                            }
                            DecodedKey::Unicode('w') | DecodedKey::Unicode('W') => {
                                self.window_manager.add_window(18, 5, 44, 14, "Window");
                                self.render_desktop();
                                self.window_manager.render_all();
                                self.render_context_menu();
                                self.render_cursor(self.last_mouse_x, self.last_mouse_y);
                            }
                            DecodedKey::RawKey(KeyCode::Escape) => {
                                if self.context_menu.is_some() {
                                    self.context_menu = None;
                                    self.render_desktop();
                                    self.window_manager.render_all();
                                    self.render_cursor(self.last_mouse_x, self.last_mouse_y);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Yield to allow other async tasks to run
            crate::task::yield_now().await;
        }
    }
}

/// Main desktop loop
pub async fn run_desktop_environment() -> IconAction {
    let mut desktop = Desktop::new();
    desktop.run().await
}
