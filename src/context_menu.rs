use alloc::vec::Vec;
use crate::vga_buffer::Color;
use crate::graphics::text_graphics::{draw_filled_box, draw_box, write_at};

const SCREEN_W: usize = 80;
const SCREEN_H: usize = 25;

#[derive(Clone)]
pub enum MenuActionKind {
    NewWindow,
    CloseWindow(u8),
    Refresh,
}

#[derive(Clone)]
pub struct MenuItem {
    pub label: &'static str,
    pub action: MenuActionKind,
}

impl MenuItem {
    pub fn new(label: &'static str, action: MenuActionKind) -> Self {
        MenuItem { label, action }
    }
}

pub struct ContextMenu {
    pub x: usize,
    pub y: usize,
    items: Vec<MenuItem>,
    hovered: Option<usize>,
    pub width: usize,
    pub height: usize,
}

impl ContextMenu {
    pub fn new(x: usize, y: usize, items: Vec<MenuItem>) -> Self {
        let max_label = items.iter().map(|i| i.label.len()).max().unwrap_or(0);
        let width = max_label + 4;
        let height = items.len() + 2;
        let x = x.min(SCREEN_W.saturating_sub(width));
        let y = y.min(SCREEN_H.saturating_sub(height));
        ContextMenu { x, y, items, hovered: None, width, height }
    }

    pub fn item_at(&self, mx: i16, my: i16) -> Option<usize> {
        if mx < 0 || my < 0 {
            return None;
        }
        let xu = mx as usize;
        let yu = my as usize;
        if xu < self.x + 1 || xu >= self.x + self.width - 1 {
            return None;
        }
        if yu < self.y + 1 || yu > self.y + self.items.len() {
            return None;
        }
        let idx = yu - self.y - 1;
        if idx < self.items.len() { Some(idx) } else { None }
    }

    pub fn action_at(&self, mx: i16, my: i16) -> Option<MenuActionKind> {
        self.item_at(mx, my).map(|i| self.items[i].action.clone())
    }

    pub fn update_hover(&mut self, mx: i16, my: i16) -> bool {
        let new_hover = self.item_at(mx, my);
        if new_hover != self.hovered {
            self.hovered = new_hover;
            true
        } else {
            false
        }
    }

    pub fn render(&self) {
        draw_filled_box(self.x, self.y, self.width, self.height, Color::White, Color::Black);
        draw_box(self.x, self.y, self.width, self.height, Color::White, Color::Black);

        for (i, item) in self.items.iter().enumerate() {
            let item_y = self.y + 1 + i;
            let hovered = self.hovered == Some(i);
            let (fg, bg) = if hovered {
                (Color::Black, Color::LightGray)
            } else {
                (Color::LightGray, Color::Black)
            };
            draw_filled_box(self.x + 1, item_y, self.width - 2, 1, fg, bg);
            write_at(self.x + 2, item_y, item.label, fg, bg);
        }
    }
}
