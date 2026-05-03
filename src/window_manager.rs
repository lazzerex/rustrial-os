use alloc::{vec::Vec, string::String};
use alloc::string::ToString;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::vga_buffer::Color;
use crate::graphics::text_graphics::{draw_filled_box, draw_double_box, write_at};

const SCREEN_W: usize = 80;
const SCREEN_H: usize = 25;
const WIN_MIN_W: usize = 20;
const WIN_MIN_H: usize = 5;
const WIN_Y_MIN: usize = 1;
const WIN_Y_MAX_OFFSET: usize = 1;

#[derive(Clone)]
pub struct FsEntry {
    pub full_path: String,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Clone)]
pub enum WindowContent {
    Empty,
    FileManager {
        cwd: String,
        entries: Vec<FsEntry>,
        selected: usize,
        scroll: usize,
    },
    Settings,
    Shell {
        input: String,
        output: Vec<String>,
        cwd: String,
        history: Vec<String>,
        history_idx: usize,
        scroll: usize,
    },
}

impl WindowContent {
    fn read_dir(path: &str) -> Vec<FsEntry> {
        use crate::fs;
        use crate::fs::FileSystem;
        let mut entries = Vec::new();
        if path != "/" {
            let parent = path
                .rsplit_once('/')
                .map(|(p, _)| if p.is_empty() { "/" } else { p })
                .unwrap_or("/");
            entries.push(FsEntry {
                full_path: String::from(parent),
                name: String::from(".."),
                is_dir: true,
            });
        }
        if let Some(fs_ref) = fs::root_fs() {
            let fs = fs_ref.lock();
            if let Ok(listing) = fs.list_dir(path) {
                for entry_path in listing {
                    let display_name = if path == "/" {
                        let trimmed = entry_path.trim_start_matches('/');
                        if trimmed.contains('/') {
                            continue;
                        }
                        trimmed.to_string()
                    } else {
                        entry_path.rsplit('/').next().unwrap_or(&entry_path).to_string()
                    };
                    let is_dir = fs.is_dir(&entry_path);
                    entries.push(FsEntry { full_path: entry_path, name: display_name, is_dir });
                }
            }
        }
        entries
    }
}

#[derive(Clone)]
pub struct Window {
    pub id: u8,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub title: String,
    z_order: u8,
    pub content: WindowContent,
}

pub struct WindowManager {
    windows: Vec<Window>,
    focus_id: Option<u8>,
    drag_id: Option<u8>,
    drag_offset_x: i16,
    drag_offset_y: i16,
    resize_id: Option<u8>,
    resize_start_mx: i16,
    resize_start_my: i16,
    resize_start_w: usize,
    resize_start_h: usize,
    next_id: u8,
}

impl WindowManager {
    pub fn new() -> Self {
        WindowManager {
            windows: Vec::new(),
            focus_id: None,
            drag_id: None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            resize_id: None,
            resize_start_mx: 0,
            resize_start_my: 0,
            resize_start_w: 0,
            resize_start_h: 0,
            next_id: 0,
        }
    }

    pub fn add_window(&mut self, x: usize, y: usize, w: usize, h: usize, title: &str) -> u8 {
        self.create_window(x, y, w, h, title, WindowContent::Empty)
    }

    pub fn add_file_manager(&mut self, x: usize, y: usize, w: usize, h: usize, path: &str) -> u8 {
        let content = WindowContent::FileManager {
            cwd: String::from(path),
            entries: WindowContent::read_dir(path),
            selected: 0,
            scroll: 0,
        };
        let title = alloc::format!("Files: {}", path);
        self.create_window(x, y, w, h, &title, content)
    }

    pub fn add_settings_window(&mut self) -> u8 {
        self.create_window(22, 4, 36, 12, "Settings", WindowContent::Settings)
    }

    pub fn add_shell_window(&mut self) -> u8 {
        let mut output = Vec::new();
        output.push(String::from("RustrialOS Shell  type 'help' for commands"));
        let content = WindowContent::Shell {
            input: String::new(),
            output,
            cwd: String::from("/"),
            history: Vec::new(),
            history_idx: 0,
            scroll: 0,
        };
        self.create_window(10, 2, 58, 20, "Shell", content)
    }

    fn create_window(&mut self, x: usize, y: usize, w: usize, h: usize, title: &str, content: WindowContent) -> u8 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let max_z = self.windows.iter().map(|win| win.z_order).max().unwrap_or(0);
        let w = w.max(WIN_MIN_W).min(SCREEN_W);
        let h = h.max(WIN_MIN_H).min(SCREEN_H.saturating_sub(WIN_Y_MIN + WIN_Y_MAX_OFFSET));
        let x = x.min(SCREEN_W.saturating_sub(w));
        let y = y.clamp(WIN_Y_MIN, SCREEN_H.saturating_sub(h + WIN_Y_MAX_OFFSET));
        self.windows.push(Window {
            id,
            x,
            y,
            w,
            h,
            title: String::from(title),
            z_order: max_z.wrapping_add(1),
            content,
        });
        self.focus_id = Some(id);
        id
    }

    pub fn close_window(&mut self, id: u8) {
        self.windows.retain(|w| w.id != id);
        if self.focus_id == Some(id) {
            self.focus_id = self.windows.iter().max_by_key(|w| w.z_order).map(|w| w.id);
        }
        if self.drag_id == Some(id) {
            self.drag_id = None;
        }
        if self.resize_id == Some(id) {
            self.resize_id = None;
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn render_all(&self) {
        let mut order: Vec<usize> = (0..self.windows.len()).collect();
        order.sort_by_key(|&i| self.windows[i].z_order);
        for i in order {
            self.render_window(&self.windows[i]);
        }
    }

    fn render_window(&self, win: &Window) {
        let focused = self.focus_id == Some(win.id);
        let title_bg = if focused { Color::Blue } else { Color::DarkGray };

        if win.h > 2 && win.w > 2 {
            draw_filled_box(win.x + 1, win.y + 1, win.w - 2, win.h - 2, Color::LightGray, Color::Black);
        }

        draw_double_box(win.x, win.y, win.w, win.h, Color::LightGray, Color::Black);

        draw_filled_box(win.x, win.y, win.w, 1, Color::White, title_bg);

        let max_title = win.w.saturating_sub(6);
        let title_disp = if win.title.len() > max_title { &win.title[..max_title] } else { &win.title };
        write_at(win.x + 1, win.y, title_disp, Color::White, title_bg);

        if win.w >= 4 {
            write_at(win.x + win.w - 4, win.y, "[X]", Color::Yellow, Color::Red);
        }

        // Resize handle: yellow bottom-right corner when focused
        if focused && win.h >= 2 && win.w >= 2 {
            crate::vga_buffer::write_char_at(
                win.x + win.w - 1,
                win.y + win.h - 1,
                0xBC, // ╝ in VGA CP437, yellow = draggable
                Color::Yellow,
                Color::Black,
            );
        }

        if win.h > 2 && win.w > 2 {
            let cx = win.x + 1;
            let cy = win.y + 1;
            let cw = win.w - 2;
            let ch = win.h - 2;
            self.render_content(win, cx, cy, cw, ch, focused);
        }
    }

    fn render_content(&self, win: &Window, cx: usize, cy: usize, cw: usize, ch: usize, focused: bool) {
        match &win.content {
            WindowContent::Empty => {}
            WindowContent::Settings => {
                self.render_settings_content(cx, cy, cw, ch, focused);
            }
            WindowContent::FileManager { cwd, entries, selected, scroll } => {
                // First row: current path
                let cwd_disp = if cwd.len() > cw { &cwd[cwd.len() - cw..] } else { cwd.as_str() };
                draw_filled_box(cx, cy, cw, 1, Color::Yellow, Color::DarkGray);
                write_at(cx, cy, cwd_disp, Color::Yellow, Color::DarkGray);

                let list_h = ch.saturating_sub(1);
                let start = *scroll;
                let end = (start + list_h).min(entries.len());

                for i in 0..list_h {
                    let row = cy + 1 + i;
                    let entry_idx = start + i;
                    if entry_idx < end {
                        let entry = &entries[entry_idx];
                        let is_selected = focused && (entry_idx == *selected);
                        let (fg, bg) = if is_selected {
                            (Color::Black, Color::LightGray)
                        } else if entry.is_dir {
                            (Color::LightBlue, Color::Black)
                        } else {
                            (Color::White, Color::Black)
                        };
                        let prefix = if entry.is_dir { "[D] " } else { "    " };
                        let name_space = cw.saturating_sub(prefix.len());
                        let name_disp = if entry.name.len() > name_space {
                            &entry.name[..name_space]
                        } else {
                            &entry.name
                        };
                        draw_filled_box(cx, row, cw, 1, fg, bg);
                        let line = alloc::format!("{}{}", prefix, name_disp);
                        write_at(cx, row, &line, fg, bg);
                    } else {
                        draw_filled_box(cx, row, cw, 1, Color::Black, Color::Black);
                    }
                }

                // Scroll indicators
                if *scroll > 0 {
                    write_at(cx + cw - 1, cy + 1, "^", Color::DarkGray, Color::Black);
                }
                if end < entries.len() {
                    write_at(cx + cw - 1, cy + list_h, "v", Color::DarkGray, Color::Black);
                }
            }
            WindowContent::Shell { .. } => {
                self.render_shell_content(win, cx, cy, cw, ch);
            }
        }
    }

    fn render_shell_content(&self, win: &Window, cx: usize, cy: usize, cw: usize, ch: usize) {
        let (input, output, cwd, scroll) = match &win.content {
            WindowContent::Shell { input, output, cwd, scroll, .. } => (input, output, cwd, *scroll),
            _ => return,
        };
        if ch == 0 { return; }

        let output_h = ch.saturating_sub(1);
        let total = output.len();
        let end_idx = total.saturating_sub(scroll);
        let start_idx = end_idx.saturating_sub(output_h);

        for i in 0..output_h {
            let row = cy + i;
            draw_filled_box(cx, row, cw, 1, Color::LightGreen, Color::Black);
            let line_idx = start_idx + i;
            if line_idx < end_idx && line_idx < total {
                let line = &output[line_idx];
                let disp = if line.len() > cw { &line[..cw] } else { line.as_str() };
                write_at(cx, row, disp, Color::LightGreen, Color::Black);
            }
        }

        let prompt_row = cy + ch - 1;
        let full_line = alloc::format!("{}> {}", cwd, input);
        draw_filled_box(cx, prompt_row, cw, 1, Color::Black, Color::Black);
        let disp = if full_line.len() > cw { &full_line[full_line.len() - cw..] } else { full_line.as_str() };
        write_at(cx, prompt_row, disp, Color::Yellow, Color::Black);
    }

    fn render_settings_content(&self, cx: usize, cy: usize, cw: usize, _ch: usize, _focused: bool) {
        let section = |y: usize, label: &str| {
            draw_filled_box(cx, y, cw, 1, Color::Yellow, Color::DarkGray);
            write_at(cx + 1, y, label, Color::Yellow, Color::DarkGray);
        };

        // Mouse speed section
        section(cy, "Mouse Speed");
        let sens = crate::task::mouse::get_sensitivity();
        let speed_str = alloc::format!("  [<]  {:2}  [>]", sens);
        draw_filled_box(cx, cy + 1, cw, 1, Color::White, Color::Black);
        write_at(cx, cy + 1, &speed_str, Color::White, Color::Black);
        draw_filled_box(cx, cy + 2, cw, 1, Color::Black, Color::Black);

        // Theme section
        section(cy + 3, "Theme");
        let theme = crate::theme::get();
        draw_filled_box(cx, cy + 4, cw, 1, Color::Black, Color::Black);
        let themes = [("Cyan ", 0u8), ("Dark ", 1), ("Night", 2)];
        let mut tx = cx + 1;
        for (label, id) in themes.iter() {
            let (fg, bg) = if *id == theme {
                (Color::Black, Color::Yellow)
            } else {
                (Color::White, Color::DarkGray)
            };
            let btn = alloc::format!("[{}]", label);
            write_at(tx, cy + 4, &btn, fg, bg);
            tx += 8;
        }
        draw_filled_box(cx, cy + 5, cw, 1, Color::Black, Color::Black);

        // Time section
        section(cy + 6, "Time  (UTC+7)");
        use crate::native_ffi;
        let mut dt = native_ffi::DateTime::read();
        let mut hour = dt.hour as i16 + 7;
        if hour >= 24 {
            hour -= 24;
        }
        dt.hour = hour as u8;
        let time_str = alloc::format!("  {:02}:{:02}:{:02}  {}", dt.hour, dt.minute, dt.second, dt.month_str());
        draw_filled_box(cx, cy + 7, cw, 1, Color::White, Color::Black);
        write_at(cx, cy + 7, &time_str, Color::LightCyan, Color::Black);
    }

    /// Returns true if a window consumed the click (focus, drag, close, resize, or content).
    pub fn on_mouse_down(&mut self, mx: i16, my: i16) -> bool {
        if mx < 0 || my < 0 {
            return false;
        }
        let mx_u = mx as usize;
        let my_u = my as usize;

        let hit = self.windows.iter().enumerate()
            .filter(|(_, w)| mx_u >= w.x && mx_u < w.x + w.w && my_u >= w.y && my_u < w.y + w.h)
            .max_by_key(|(_, w)| w.z_order)
            .map(|(i, _)| i);

        if let Some(idx) = hit {
            let (id, win_x, win_y, win_w, win_h) = {
                let w = &self.windows[idx];
                (w.id, w.x, w.y, w.w, w.h)
            };

            // Close button
            if my_u == win_y && mx_u >= win_x + win_w.saturating_sub(4) {
                self.close_window(id);
                return true;
            }

            // Title bar drag
            if my_u == win_y {
                self.drag_id = Some(id);
                self.drag_offset_x = mx - win_x as i16;
                self.drag_offset_y = my - win_y as i16;
                self.bring_to_front(id);
                self.focus_id = Some(id);
                return true;
            }

            // Resize handle: bottom-right corner
            if my_u == win_y + win_h - 1 && mx_u == win_x + win_w - 1 {
                self.resize_id = Some(id);
                self.resize_start_mx = mx;
                self.resize_start_my = my;
                self.resize_start_w = win_w;
                self.resize_start_h = win_h;
                self.bring_to_front(id);
                self.focus_id = Some(id);
                return true;
            }

            // Content area click
            if my_u > win_y && my_u < win_y + win_h - 1 && mx_u > win_x && mx_u < win_x + win_w - 1 {
                let inner_x = mx_u - win_x - 1;
                let inner_y = my_u - win_y - 1;
                self.bring_to_front(id);
                self.focus_id = Some(id);
                self.handle_content_click(id, inner_x, inner_y);
                return true;
            }

            self.bring_to_front(id);
            self.focus_id = Some(id);
            true
        } else {
            false
        }
    }

    fn handle_content_click(&mut self, id: u8, inner_x: usize, inner_y: usize) {
        if let Some(win) = self.windows.iter().find(|w| w.id == id) {
            match win.content {
                WindowContent::Settings => {
                    self.handle_settings_click(inner_x, inner_y);
                    return;
                }
                _ => {}
            }
        }

        // File manager: inner_y 0 = cwd header, skip
        if inner_y == 0 {
            return;
        }
        let mut navigate_to: Option<String> = None;

        if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
            if let WindowContent::FileManager { ref entries, ref mut selected, ref scroll, .. } = win.content {
                let entry_idx = scroll.saturating_add(inner_y.saturating_sub(1));
                if entry_idx < entries.len() {
                    *selected = entry_idx;
                    if entries[entry_idx].is_dir {
                        navigate_to = Some(entries[entry_idx].full_path.clone());
                    }
                }
            }
        }

        if let Some(path) = navigate_to {
            let new_entries = WindowContent::read_dir(&path);
            let new_title = alloc::format!("Files: {}", path);
            if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
                win.title = new_title;
                win.content = WindowContent::FileManager {
                    cwd: path,
                    entries: new_entries,
                    selected: 0,
                    scroll: 0,
                };
            }
        }
    }

    fn handle_settings_click(&self, inner_x: usize, inner_y: usize) {
        match inner_y {
            1 => {
                // Mouse speed row: [<] at cols 2-4, [>] at cols 9-11
                let sens = crate::task::mouse::get_sensitivity();
                if inner_x >= 2 && inner_x <= 4 {
                    crate::task::mouse::set_sensitivity(sens.saturating_sub(2));
                } else if inner_x >= 11 && inner_x <= 13 {
                    crate::task::mouse::set_sensitivity(sens.saturating_add(2));
                }
            }
            4 => {
                // Theme row: [Cyan ] at 1-7, [Dark ] at 9-15, [Night] at 17-23
                if inner_x >= 1 && inner_x <= 7 {
                    crate::theme::set(0);
                } else if inner_x >= 9 && inner_x <= 15 {
                    crate::theme::set(1);
                } else if inner_x >= 17 && inner_x <= 23 {
                    crate::theme::set(2);
                }
            }
            _ => {}
        }
    }

    /// Returns true if dragging or resizing AND the window actually moved (redraw needed).
    pub fn on_mouse_move(&mut self, mx: i16, my: i16) -> bool {
        if let Some(resize_id) = self.resize_id {
            if let Some(win) = self.windows.iter_mut().find(|w| w.id == resize_id) {
                let dw = mx - self.resize_start_mx;
                let dh = my - self.resize_start_my;
                let new_w = ((self.resize_start_w as i16) + dw).max(WIN_MIN_W as i16) as usize;
                let new_h = ((self.resize_start_h as i16) + dh).max(WIN_MIN_H as i16) as usize;
                let new_w = new_w.min(SCREEN_W.saturating_sub(win.x));
                let new_h = new_h.min(SCREEN_H.saturating_sub(win.y + WIN_Y_MAX_OFFSET));
                if new_w != win.w || new_h != win.h {
                    win.w = new_w;
                    win.h = new_h;
                    return true;
                }
                return false;
            }
        }

        if let Some(drag_id) = self.drag_id {
            if let Some(win) = self.windows.iter_mut().find(|w| w.id == drag_id) {
                let new_x = (mx - self.drag_offset_x).max(0) as usize;
                let new_y = (my - self.drag_offset_y).max(0) as usize;
                let new_x = new_x.min(SCREEN_W.saturating_sub(win.w));
                let new_y = new_y.clamp(WIN_Y_MIN, SCREEN_H.saturating_sub(win.h + WIN_Y_MAX_OFFSET));
                if new_x != win.x || new_y != win.y {
                    win.x = new_x;
                    win.y = new_y;
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub fn is_dragging_or_resizing(&self) -> bool {
        self.drag_id.is_some() || self.resize_id.is_some()
    }

    pub fn focused_window_is_shell(&self) -> bool {
        self.focus_id
            .and_then(|id| self.windows.iter().find(|w| w.id == id))
            .map(|w| matches!(w.content, WindowContent::Shell { .. }))
            .unwrap_or(false)
    }

    pub fn handle_shell_key(&mut self, key: DecodedKey) -> bool {
        let id = match self.focus_id { Some(id) => id, None => return false };
        let idx = match self.windows.iter().position(|w| w.id == id) { Some(i) => i, None => return false };

        match &mut self.windows[idx].content {
            WindowContent::Shell { input, output, cwd, history, history_idx, scroll } => {
                match key {
                    DecodedKey::Unicode('\n') | DecodedKey::Unicode('\r') => {
                        let cmd = input.clone();
                        output.push(alloc::format!("{}> {}", cwd, cmd));
                        if !cmd.is_empty() {
                            if history.len() >= 50 { history.remove(0); }
                            history.push(cmd.clone());
                            *history_idx = history.len();
                            shell_exec_to_buf(&cmd, cwd, output);
                        }
                        input.clear();
                        *scroll = 0;
                        true
                    }
                    DecodedKey::Unicode('\u{0008}') | DecodedKey::RawKey(KeyCode::Backspace) => {
                        input.pop();
                        true
                    }
                    DecodedKey::Unicode(c) if !c.is_control() => {
                        input.push(c);
                        true
                    }
                    DecodedKey::RawKey(KeyCode::ArrowUp) => {
                        if *history_idx > 0 {
                            *history_idx -= 1;
                            *input = history[*history_idx].clone();
                        }
                        true
                    }
                    DecodedKey::RawKey(KeyCode::ArrowDown) => {
                        if *history_idx < history.len() {
                            *history_idx += 1;
                            if *history_idx < history.len() {
                                *input = history[*history_idx].clone();
                            } else {
                                input.clear();
                            }
                        }
                        true
                    }
                    DecodedKey::RawKey(KeyCode::PageUp) => {
                        *scroll = scroll.saturating_add(5);
                        true
                    }
                    DecodedKey::RawKey(KeyCode::PageDown) => {
                        *scroll = scroll.saturating_sub(5);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Bounds of the currently dragged/resized window, used for targeted erase.
    pub fn current_drag_bounds(&self) -> Option<(usize, usize, usize, usize)> {
        let id = self.drag_id.or(self.resize_id)?;
        let win = self.windows.iter().find(|w| w.id == id)?;
        Some((win.x, win.y, win.w, win.h))
    }

    pub fn on_mouse_up(&mut self) {
        self.drag_id = None;
        self.resize_id = None;
    }

    pub fn get_windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn is_focused(&self, id: u8) -> bool {
        self.focus_id == Some(id)
    }

    pub fn focus_and_raise(&mut self, id: u8) {
        self.bring_to_front(id);
        self.focus_id = Some(id);
    }

    pub fn topmost_window_at(&self, mx: i16, my: i16) -> Option<u8> {
        if mx < 0 || my < 0 {
            return None;
        }
        let mx_u = mx as usize;
        let my_u = my as usize;
        self.windows.iter()
            .filter(|w| mx_u >= w.x && mx_u < w.x + w.w && my_u >= w.y && my_u < w.y + w.h)
            .max_by_key(|w| w.z_order)
            .map(|w| w.id)
    }

    pub fn render_windows_overlapping(&self, x: usize, y: usize, w: usize, h: usize) {
        let x_end = x + w;
        let y_end = y + h;
        let mut order: Vec<usize> = (0..self.windows.len()).collect();
        order.sort_by_key(|&i| self.windows[i].z_order);
        for i in order {
            let win = &self.windows[i];
            if win.x < x_end && win.x + win.w > x && win.y < y_end && win.y + win.h > y {
                self.render_window(win);
            }
        }
    }

    pub fn is_point_over_window(&self, mx: i16, my: i16) -> bool {
        if mx < 0 || my < 0 {
            return false;
        }
        let mx_u = mx as usize;
        let my_u = my as usize;
        self.windows.iter().any(|w| {
            mx_u >= w.x && mx_u < w.x + w.w && my_u >= w.y && my_u < w.y + w.h
        })
    }

    fn bring_to_front(&mut self, id: u8) {
        let max_z = self.windows.iter().map(|w| w.z_order).max().unwrap_or(0);
        if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
            win.z_order = max_z.wrapping_add(1);
        }
    }
}

fn shell_exec_to_buf(cmd: &str, cwd: &mut String, output: &mut Vec<String>) {
    use crate::fs::FileSystem;
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    if parts.is_empty() { return; }
    match parts[0] {
        "help" => {
            output.push(String::from("Commands:"));
            output.push(String::from("  help  echo  ls  cat  cd  pwd  mkdir  touch  clear"));
            output.push(String::from("  run  fetch  netinfo  pciinfo  arp  ifconfig  dmastat  tcptest"));
            output.push(String::from("  (ping/dhcp-acquire/ntp-sync/http-get: use desktop Shell)"));
        }
        "echo" => { output.push(parts[1..].join(" ")); }
        "clear" | "cls" => { output.clear(); }
        "pwd" => { output.push(cwd.clone()); }
        "cd" => {
            let target = if parts.len() < 2 { "/" } else { parts[1] };
            let new_path = shell_resolve_path(cwd, target);
            if new_path == "/" {
                *cwd = new_path;
                return;
            }
            if let Some(fs) = crate::fs::root_fs() {
                let fs = fs.lock();
                if fs.is_dir(&new_path) {
                    *cwd = new_path;
                } else {
                    output.push(alloc::format!("cd: not a directory: {}", target));
                }
            }
        }
        "ls" => {
            let path = if parts.len() > 1 { shell_resolve_path(cwd, parts[1]) } else { cwd.clone() };
            if let Some(fs) = crate::fs::root_fs() {
                let fs = fs.lock();
                match fs.list_dir(&path) {
                    Ok(entries) => {
                        if entries.is_empty() { output.push(String::from("(empty)")); }
                        for entry in entries {
                            let is_dir = fs.is_dir(&entry);
                            let name = entry.rsplit('/').next().unwrap_or(&entry).to_string();
                            output.push(alloc::format!("{} {}", if is_dir { "[D]" } else { "[F]" }, name));
                        }
                    }
                    Err(_) => output.push(alloc::format!("ls: cannot access '{}'", path)),
                }
            }
        }
        "cat" => {
            if parts.len() < 2 { output.push(String::from("Usage: cat <file>")); return; }
            let path = shell_resolve_path(cwd, parts[1]);
            if let Some(fs) = crate::fs::root_fs() {
                let fs = fs.lock();
                match fs.read_file(&path) {
                    Ok(content) => match core::str::from_utf8(&content) {
                        Ok(text) => { for line in text.lines() { output.push(line.to_string()); } }
                        Err(_) => output.push(alloc::format!("(binary, {} bytes)", content.len())),
                    },
                    Err(_) => output.push(alloc::format!("cat: {}: no such file", parts[1])),
                }
            }
        }
        "mkdir" => {
            if parts.len() < 2 { output.push(String::from("Usage: mkdir <dir>")); return; }
            let path = shell_resolve_path(cwd, parts[1]);
            if let Some(fs) = crate::fs::root_fs() {
                let mut fs = fs.lock();
                match fs.create_dir(&path) {
                    Ok(_) => output.push(alloc::format!("created '{}'", path)),
                    Err(e) => output.push(alloc::format!("mkdir: {:?}", e)),
                }
            }
        }
        "touch" => {
            if parts.len() < 2 { output.push(String::from("Usage: touch <file>")); return; }
            let path = shell_resolve_path(cwd, parts[1]);
            if let Some(fs) = crate::fs::root_fs() {
                let mut fs = fs.lock();
                match fs.create_file(&path, b"") {
                    Ok(_) => output.push(alloc::format!("created '{}'", path)),
                    Err(e) => output.push(alloc::format!("touch: {:?}", e)),
                }
            }
        }
        "run" => {
            if parts.len() < 2 { output.push(String::from("Usage: run <script>")); return; }
            let path = if parts[1].starts_with('/') {
                parts[1].to_string()
            } else {
                alloc::format!("/scripts/{}", parts[1])
            };
            if let Some(fs) = crate::fs::root_fs() {
                let fs = fs.lock();
                match fs.read_file(&path) {
                    Ok(content) => match core::str::from_utf8(&content) {
                        Ok(text) => {
                            output.push(alloc::format!("Running: {}", path));
                            match crate::rustrial_script::run(text) {
                                Ok(_) => output.push(String::from("Script completed")),
                                Err(e) => output.push(alloc::format!("Script error: {}", e)),
                            }
                        }
                        Err(_) => output.push(String::from("Error: file is not valid UTF-8")),
                    },
                    Err(_) => output.push(alloc::format!("Error: cannot read '{}'", path)),
                }
            }
        }
        "rustrialfetch" | "fetch" => {
            use crate::native_ffi;
            let cpu = native_ffi::CpuInfo::get();
            let dt = native_ffi::DateTime::read();
            let pci = native_ffi::enumerate_pci_devices();
            let scripts = if let Some(fs) = crate::fs::root_fs() {
                fs.lock().list_dir("/scripts").map(|e| e.len()).unwrap_or(0)
            } else { 0 };
            output.push(String::from("OS:          RustrialOS v0.1"));
            output.push(String::from("Kernel:      Rust bare-metal"));
            output.push(alloc::format!("CPU:         {}", cpu.brand_str()));
            output.push(alloc::format!("Date:        {}", dt));
            output.push(alloc::format!("PCI Devices: {}", pci.len()));
            output.push(alloc::format!("Scripts:     {}", scripts));
        }
        "netinfo" => {
            use crate::drivers::net::{get_network_device, has_network_device};
            if has_network_device() {
                let dev_mutex = get_network_device();
                let dev = dev_mutex.lock();
                if let Some(ref d) = *dev {
                    let mac = d.mac_address();
                    output.push(alloc::format!("Device: {}", d.device_name()));
                    output.push(alloc::format!("MAC:    {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
                    output.push(alloc::format!("Ready:  {}", d.is_ready()));
                }
            } else {
                output.push(String::from("No network device found"));
            }
            let cfg = crate::net::stack::get_network_config();
            output.push(alloc::format!("IP:      {}", cfg.ip_addr));
            output.push(alloc::format!("Netmask: {}", cfg.netmask));
            if let Some(gw) = cfg.gateway {
                output.push(alloc::format!("Gateway: {}", gw));
            }
        }
        "pciinfo" => {
            use crate::native_ffi;
            let devices = native_ffi::enumerate_pci_devices();
            output.push(alloc::format!("{} PCI devices:", devices.len()));
            for (i, dev) in devices.iter().enumerate() {
                output.push(alloc::format!("  #{}: {}", i + 1, dev));
            }
        }
        "arp" => {
            use crate::net::arp::{arp_cache, format_mac};
            if parts.len() > 1 && parts[1] == "clear" {
                arp_cache().clear();
                output.push(String::from("ARP cache cleared"));
                return;
            }
            let entries = arp_cache().entries();
            if entries.is_empty() {
                output.push(String::from("ARP cache: (empty)"));
            } else {
                output.push(String::from("ARP cache:"));
                for (ip, mac, _) in &entries {
                    output.push(alloc::format!("  {} -> {}", ip, format_mac(mac)));
                }
            }
        }
        "ifconfig" => {
            use core::net::Ipv4Addr;
            use crate::net::stack::{NetworkConfig, set_network_config, get_network_config};
            if parts.len() == 1 {
                let cfg = get_network_config();
                output.push(alloc::format!("IP:      {}", cfg.ip_addr));
                output.push(alloc::format!("Netmask: {}", cfg.netmask));
                if let Some(gw) = cfg.gateway {
                    output.push(alloc::format!("Gateway: {}", gw));
                }
            } else if parts.len() >= 3 {
                match (parts[1].parse::<Ipv4Addr>(), parts[2].parse::<Ipv4Addr>()) {
                    (Ok(ip), Ok(mask)) => {
                        let gw = if parts.len() >= 4 { parts[3].parse::<Ipv4Addr>().ok() } else { None };
                        set_network_config(NetworkConfig::new(ip, mask, gw));
                        output.push(String::from("Network config updated"));
                    }
                    _ => output.push(String::from("Usage: ifconfig [<ip> <netmask> [gateway]]")),
                }
            } else {
                output.push(String::from("Usage: ifconfig [<ip> <netmask> [gateway]]"));
            }
        }
        "dmastat" => {
            let s = crate::memory::dma::get_dma_stats();
            output.push(alloc::format!("Allocated: {} KB", s.total_allocated / 1024));
            output.push(alloc::format!("Current:   {} KB", s.current_usage / 1024));
            output.push(alloc::format!("Peak:      {} KB", s.peak_usage / 1024));
            output.push(alloc::format!("Pool hits: {}  misses: {}", s.pool_hits, s.pool_misses));
        }
        "tcptest" => {
            use core::net::Ipv4Addr;
            use crate::net::tcp::{TcpConnection, TcpSocketId, TcpState};
            let sid = TcpSocketId {
                local_addr: Ipv4Addr::new(10,0,2,15), local_port: 8080,
                remote_addr: Ipv4Addr::new(10,0,2,2), remote_port: 80,
            };
            let conn = TcpConnection::new(sid);
            output.push(if conn.state == TcpState::Closed {
                String::from("[OK] Initial state: CLOSED")
            } else {
                String::from("[FAIL] Initial state incorrect")
            });
            output.push(alloc::format!("CWND: {}  SSThresh: {}", conn.cwnd, conn.ssthresh));
            let mut conn2 = TcpConnection::new(sid);
            match conn2.connect() {
                Ok(_) => {
                    output.push(alloc::format!("[OK] ISN: {}", conn2.initial_send_seq));
                    output.push(if conn2.state == TcpState::SynSent {
                        String::from("[OK] State: SYN_SENT")
                    } else {
                        String::from("[FAIL] State transition failed")
                    });
                }
                Err(_) => output.push(String::from("[FAIL] connect() failed")),
            }
        }
        "ping" | "dhcp-acquire" | "ntp-sync" | "http-get" => {
            output.push(alloc::format!("'{}' requires async networking", parts[0]));
            output.push(String::from("Use the desktop Shell icon for async commands"));
        }
        _ => { output.push(alloc::format!("'{}': unknown command (type 'help')", parts[0])); }
    }
}

fn shell_resolve_path(cwd: &str, path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else if path == ".." {
        if cwd == "/" { return String::from("/"); }
        match cwd.rsplit_once('/') {
            Some((parent, _)) => if parent.is_empty() { String::from("/") } else { parent.to_string() },
            None => String::from("/"),
        }
    } else if path == "." {
        cwd.to_string()
    } else if cwd == "/" {
        alloc::format!("/{}", path)
    } else {
        alloc::format!("{}/{}", cwd, path)
    }
}
