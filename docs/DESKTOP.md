# Desktop GUI Environment

## Overview

The Rustrial OS now features a graphical desktop environment that provides a modern, OS-like experience with mouse support, clickable icons, and window management. This replaces the direct boot-to-menu approach with a desktop-first interface.

## Features

### Desktop Environment
- **Visual Desktop**: A cyan-gradient desktop background with a title bar and status bar
- **Icon Grid**: Desktop icons arranged in an organized grid layout
- **System Clock**: Real-time clock display in the title bar (via RTC)
- **Mouse Cursor**: Full PS/2 mouse support with visible cursor rendering
- **Keyboard Navigation**: Arrow keys can be used to navigate between icons

### Mouse Support
- **PS/2 Mouse Driver**: Complete implementation with packet parsing
- **Cursor Tracking**: Real-time cursor position updates in 80x25 text mode
- **Click Detection**: Left and right button support with double-click detection
- **Icon Highlighting**: Icons highlight when the mouse hovers over them

### Desktop Icons

The desktop includes four main icons:

1. **Main Menu** - Opens the full interactive menu system with all options
2. **System** - Quick access to system information and hardware summary
3. **Scripts** - Direct access to RustrialScript features
4. **Hardware** - View detailed hardware information (CPU, RTC, PCI devices)

### User Interaction

#### Mouse Control
- Move the mouse to hover over icons (they will highlight in light blue)
- Double-click an icon to launch the associated application
- Click and drag is not yet implemented but cursor movement is fully tracked

#### Keyboard Control
- **Arrow Left/Right**: Navigate between icons
- **Enter/Space**: Activate the selected icon
- **ESC**: Return from applications to the desktop

#### Menu Integration
- When you launch the Main Menu icon, you get the full menu experience
- The menu now includes option **[0] Exit to Desktop** to return
- All menu features work as before: scripts, graphics demo, hardware info, etc.
- Pressing **0** or selecting Exit returns you to the desktop

## Technical Details

### Architecture

```
Boot Splash → Desktop Environment → Applications
                     ↑                    ↓
                     └────────────────────┘
                         (User can return)
```

### Modules

#### `src/task/mouse.rs`
- PS/2 mouse packet parsing
- Button state tracking
- Cursor position management
- Hardware initialization (IRQ 12)

#### `src/desktop.rs`
- Desktop icon rendering
- Mouse cursor rendering
- Icon selection and click handling
- Event loop for desktop interaction
- Application launching

#### `src/interrupts.rs`
- Mouse interrupt handler (IRQ 12)
- Integration with existing keyboard interrupt

#### `src/rustrial_menu.rs`
- Updated to return boolean (exit to desktop flag)
- New option [0] to exit to desktop
- Seamless integration with desktop environment

### Dependencies

```toml
embedded-graphics = { version = "0.8", default-features = false }
tinybmp = { version = "0.5", default-features = false }
```

**Note**: Currently embedded-graphics is included for future expansion but the current implementation uses the existing VGA text mode graphics.

## Boot Sequence

1. **Bootloader** loads the kernel
2. **Kernel initialization** (GDT, IDT, PIC, heap)
3. **Boot splash screen** with animated progress
4. **Mouse hardware initialization** (PS/2 controller)
5. **Desktop environment** launches
6. **User interaction** begins

## Usage Guide

### Launching Applications

1. **Boot the OS** - Wait for the boot splash to complete
2. **See the desktop** - Four icons will appear on a cyan background
3. **Select an icon**:
   - Move mouse over an icon (it will highlight)
   - Double-click to launch
   - OR use arrow keys and press Enter

### Navigating the Menu

1. **Launch "Main Menu" icon**
2. **Choose an option** (1-5):
   - [1] Normal operation mode (keyboard echo)
   - [2] RustrialScript (demo or browser)
   - [3] Graphics Demo
   - [4] Hardware Info submenu
   - [5] Help
   - [0] **Exit to Desktop** ← New!
3. **Press 0** to return to desktop at any time

### Hardware Information

The **Hardware** icon provides quick access to:
- CPU brand string and features
- Real-time clock (RTC) date/time
- PCI device enumeration
- All detected via native C/Assembly FFI

## Future Enhancements

### Planned Features
- **Window System**: Movable, resizable windows for applications
- **Task Bar**: Quick launch area and system tray
- **File Manager**: Visual file browser for the RAM filesystem
- **Settings Panel**: Configure mouse speed, colors, time
- **Application Windows**: Run multiple apps side-by-side

### Potential Improvements
- **Right-click context menus**: Additional actions per icon
- **Drag and drop**: Move icons or files
- **Icon customization**: User-defined icon positions
- **Themes**: Multiple color schemes
- **Graphics mode**: Switch from text mode to VGA 320x200 or VESA

## Technical Implementation Notes

### Mouse Packet Format
PS/2 mouse sends 3-byte packets:
```
Byte 0: [Y_OV][X_OV][Y_S][X_S][1][MB][RB][LB]
Byte 1: X movement (8-bit signed)
Byte 2: Y movement (8-bit signed)
```

### Cursor Position
- Stored in atomic variables for thread-safe access
- Updated by mouse interrupt handler
- Bounded to screen dimensions (0-79 x, 0-24 y)
- Movement scaled by 2x for better control

### Double-Click Detection
- Timer-based: 50 polling cycles (≈1 second)
- Tracks last clicked icon
- Resets on timeout or different icon click

### Icon Grid Layout
Icons are positioned at:
- Row 4, columns: 5, 20, 35, 50
- Each icon is 12 chars wide × 5 lines tall
- Labels centered below icon symbols

## Troubleshooting

### Mouse Not Working
- Ensure PS/2 mouse is connected before boot
- Check QEMU/emulator mouse integration settings
- In QEMU, click in window to capture mouse

### No Cursor Visible
- Mouse hardware initialization may have failed
- Try `Ctrl+Alt+G` to release/recapture mouse (QEMU)
- Restart the OS

### Icons Not Responding
- Try keyboard navigation (arrow keys + Enter)
- Check if mouse interrupts are being generated
- Ensure double-clicking (single clicks won't launch)

### Can't Return to Desktop
- Press **0** in the Main Menu
- Or use **ESC** to navigate back through submenus

## Code Examples

### Launching the Desktop
```rust
// From main.rs
async fn desktop_loop() {
    loop {
        let action = run_desktop_environment().await;
        match action {
            IconAction::OpenMenu => { /* ... */ }
            IconAction::SystemInfo => { /* ... */ }
            // ... handle other actions
        }
    }
}
```

### Creating an Icon
```rust
let icon = DesktopIcon::new(
    5,              // x position
    4,              // y position
    "Main Menu",    // label
    IconAction::OpenMenu  // action when clicked
);
```

### Reading Mouse Position
```rust
use crate::task::mouse::get_position;

let (x, y) = get_position();
println!("Mouse at: {}, {}", x, y);
```

## Performance

- **Cursor Rendering**: ~1000 CPU cycles per frame
- **Icon Hover Detection**: O(n) where n = number of icons
- **Mouse Interrupt Rate**: ~100 Hz (PS/2 standard)
- **Event Loop**: Non-blocking with async/await

## Credits

- **PS/2 Mouse Driver**: Custom implementation
- **Desktop Design**: Inspired by classic OS desktop environments
- **Graphics**: Using existing Rustrial text-mode graphics library
- **Icon System**: Original design for Rustrial OS

---

**Rustrial OS Desktop Environment v0.1.0**  
*A modern desktop experience in a custom operating system*
