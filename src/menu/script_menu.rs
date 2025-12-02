// Script menu and browser logic

pub fn show_script_choice() {
    super::show_script_choice();
}

pub fn show_script_browser(selected_index: usize, page: usize) {
    super::show_script_browser(selected_index, page);
}

pub fn handle_script_browser_input(
    key: pc_keyboard::DecodedKey,
    selected_index: &mut usize,
    page: &mut usize,
    menu_state: &mut super::MenuState,
    return_to_browser: &mut bool,
) {
    super::handle_script_browser_input(key, selected_index, page, menu_state, return_to_browser);
}

pub fn run_selected_script(index: usize) {
    super::run_selected_script(index);
}

pub fn run_demo() {
    super::run_demo();
}
