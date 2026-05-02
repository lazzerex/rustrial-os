use alloc::{vec::Vec, string::String};
use crate::vga_buffer::Color;
use crate::graphics::text_graphics::{draw_filled_box, draw_double_box, write_at};

const SCREEN_W: usize = 80;
const SCREEN_H: usize = 25;
const WIN_MIN_W: usize = 20;
const WIN_MIN_H: usize = 5;
const WIN_Y_MIN: usize = 1;
const WIN_Y_MAX_OFFSET: usize = 1;

#[derive(Clone)]
pub struct Window {
    pub id: u8,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub title: String,
    z_order: u8,
}

pub struct WindowManager {
    windows: Vec<Window>,
    focus_id: Option<u8>,
    drag_id: Option<u8>,
    drag_offset_x: i16,
    drag_offset_y: i16,
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
            next_id: 0,
        }
    }

    pub fn add_window(&mut self, x: usize, y: usize, w: usize, h: usize, title: &str) -> u8 {
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

        // Body interior
        if win.h > 2 && win.w > 2 {
            draw_filled_box(win.x + 1, win.y + 1, win.w - 2, win.h - 2, Color::LightGray, Color::Black);
        }

        // Border (draws over top/bottom/sides; title bar fill will overwrite row win.y)
        draw_double_box(win.x, win.y, win.w, win.h, Color::LightGray, Color::Black);

        // Title bar (overwrites top border row with solid color)
        draw_filled_box(win.x, win.y, win.w, 1, Color::White, title_bg);

        // Title text
        let max_title = win.w.saturating_sub(6);
        let title_disp = if win.title.len() > max_title { &win.title[..max_title] } else { &win.title };
        write_at(win.x + 1, win.y, title_disp, Color::White, title_bg);

        // Close button
        if win.w >= 4 {
            write_at(win.x + win.w - 4, win.y, "[X]", Color::Yellow, Color::Red);
        }
    }

    /// Returns true if a window consumed the click (focus, drag start, or close).
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
            let (id, win_x, win_y, win_w) = {
                let w = &self.windows[idx];
                (w.id, w.x, w.y, w.w)
            };

            // Close button: last 4 cols of title bar row
            if my_u == win_y && mx_u >= win_x + win_w.saturating_sub(4) {
                self.close_window(id);
                return true;
            }

            // Title bar drag
            if my_u == win_y {
                self.drag_id = Some(id);
                self.drag_offset_x = mx - win_x as i16;
                self.drag_offset_y = my - win_y as i16;
            }

            self.bring_to_front(id);
            self.focus_id = Some(id);
            true
        } else {
            false
        }
    }

    /// Returns true if actively dragging (caller should do full redraw).
    pub fn on_mouse_move(&mut self, mx: i16, my: i16) -> bool {
        if let Some(drag_id) = self.drag_id {
            if let Some(win) = self.windows.iter_mut().find(|w| w.id == drag_id) {
                let new_x = (mx - self.drag_offset_x).max(0) as usize;
                let new_y = (my - self.drag_offset_y).max(0) as usize;
                win.x = new_x.min(SCREEN_W.saturating_sub(win.w));
                win.y = new_y.clamp(WIN_Y_MIN, SCREEN_H.saturating_sub(win.h + WIN_Y_MAX_OFFSET));
                return true;
            }
        }
        false
    }

    pub fn on_mouse_up(&mut self) {
        self.drag_id = None;
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
