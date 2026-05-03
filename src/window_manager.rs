use alloc::{vec::Vec, string::String};
use alloc::string::ToString;
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
        }
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
                let inner_y = my_u - win_y - 1;
                self.bring_to_front(id);
                self.focus_id = Some(id);
                self.handle_content_click(id, inner_y);
                return true;
            }

            self.bring_to_front(id);
            self.focus_id = Some(id);
            true
        } else {
            false
        }
    }

    fn handle_content_click(&mut self, id: u8, inner_y: usize) {
        // inner_y 0 = cwd header row, skip navigation
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
