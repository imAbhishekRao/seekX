use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;
#[cfg(feature = "layer-shell")]
use gtk4_layer_shell as layer_shell;
#[cfg(feature = "layer-shell")]
use gtk4_layer_shell::LayerShell;

use crate::launcher::{Launcher, RankedApp};

const RESULT_LIMIT: usize = 9;
const WINDOW_WIDTH: i32 = 580;
const WINDOW_HEIGHT: i32 = 260;
const RESULTS_AREA_HEIGHT: i32 = 170;

#[derive(Default)]
struct UiState {
    results: Vec<RankedApp>,
}

pub fn run(launcher: Launcher) {
    let app = gtk::Application::builder()
        .application_id("com.seekx.launcher")
        .build();

    app.connect_activate(move |app| {
        build_ui(app, launcher.clone());
    });

    app.run();
}

fn build_ui(app: &gtk::Application, launcher: Launcher) {
    install_css();

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("seekX")
        .default_width(WINDOW_WIDTH)
        .default_height(WINDOW_HEIGHT)
        .resizable(false)
        .decorated(false)
        .build();
    window.add_css_class("seekx-window");
    window.set_hide_on_close(true);
    setup_layer_shell(&window);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    container.add_css_class("seekx-root");

    let entry = gtk::Entry::builder()
        .placeholder_text("Search apps or web")
        .hexpand(true)
        .build();
    entry.add_css_class("seekx-entry");

    let status = gtk::Label::new(Some("Matches: 0"));
    status.set_xalign(0.0);
    status.add_css_class("seekx-status");

    let list = gtk::ListBox::new();
    list.add_css_class("seekx-list");
    list.set_selection_mode(gtk::SelectionMode::Single);
    list.set_activate_on_single_click(false);

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .child(&list)
        .build();
    scroller.set_has_frame(false);
    scroller.set_visible(true);
    scroller.set_min_content_height(RESULTS_AREA_HEIGHT);
    scroller.set_max_content_height(RESULTS_AREA_HEIGHT);
    scroller.add_css_class("seekx-scroll");

    container.append(&entry);
    container.append(&status);
    container.append(&scroller);
    window.set_child(Some(&container));

    let state = Rc::new(RefCell::new(UiState::default()));

    {
        let state = state.clone();
        let launcher = launcher.clone();
        let list = list.clone();
        let scroller = scroller.clone();
        let window = window.clone();
        let container = container.clone();
        let status = status.clone();
        entry.connect_changed(move |entry| {
            refresh_results(
                &launcher,
                entry,
                &list,
                &scroller,
                &status,
                &container,
                &window,
                &state,
            );
        });
    }

    {
        let launcher = launcher.clone();
        let state = state.clone();
        let entry = entry.clone();
        let window = window.clone();
        list.connect_row_activated(move |_, row| {
            let idx = row.index().max(0) as usize;
            let selected = state.borrow().results.get(idx).cloned();
            if let Some(item) = selected {
                launcher.launch_app(&item.app);
                window.close();
            } else if launcher.web_search(entry.text().as_str()) {
                window.close();
            }
        });
    }

    {
        let launcher = launcher.clone();
        let state = state.clone();
        let list = list.clone();
        let entry = entry.clone();
        let entry_for_signal = entry.clone();
        let window = window.clone();
        entry_for_signal.connect_activate(move |_| {
            trigger_primary_action(&launcher, &state, &list, &entry, &window);
        });
    }

    let key_controller = gtk::EventControllerKey::new();
    {
        let launcher = launcher.clone();
        let state = state.clone();
        let list = list.clone();
        let entry = entry.clone();
        let window = window.clone();
        key_controller.connect_key_pressed(move |_, key, _, mods| match key {
            gdk::Key::Escape => {
                window.close();
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Down => {
                move_selection(&list, &state, 1);
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Up => {
                move_selection(&list, &state, -1);
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Return => {
                if mods.contains(gdk::ModifierType::ALT_MASK) {
                    let q = entry.text().to_string();
                    if launcher.web_search(&q) {
                        window.close();
                    }
                    return gtk::glib::Propagation::Stop;
                }
                gtk::glib::Propagation::Proceed
            }
            _ => gtk::glib::Propagation::Proceed,
        });
    }
    window.add_controller(key_controller);

    refresh_results(
        &launcher,
        &entry,
        &list,
        &scroller,
        &status,
        &container,
        &window,
        &state,
    );
    window.present();
    entry.grab_focus();
}

#[cfg(feature = "layer-shell")]
fn setup_layer_shell(window: &gtk::ApplicationWindow) {
    if !layer_shell::is_supported() {
        return;
    }

    window.init_layer_shell();
    window.set_layer(layer_shell::Layer::Overlay);
    window.set_keyboard_mode(layer_shell::KeyboardMode::Exclusive);
    window.set_namespace("seekx");
    center_layer_shell(window, WINDOW_HEIGHT);
}

#[cfg(not(feature = "layer-shell"))]
fn setup_layer_shell(_window: &gtk::ApplicationWindow) {}

#[cfg(feature = "layer-shell")]
fn center_layer_shell(window: &gtk::ApplicationWindow, _height: i32) {
    let display = match gdk::Display::default() {
        Some(display) => display,
        None => return,
    };

    let monitor_from_surface = window
        .surface()
        .and_then(|surface| display.monitor_at_surface(&surface));

    let monitor = monitor_from_surface.or_else(|| {
        let monitors = display.monitors();
        monitors
            .item(0)
            .and_then(|obj| obj.downcast::<gdk::Monitor>().ok())
    });

    let Some(monitor) = monitor else {
        return;
    };

    let geometry = monitor.geometry();
    let left = ((geometry.width() - WINDOW_WIDTH) / 2).max(0);
    let top = ((geometry.height() - WINDOW_HEIGHT) / 2).max(0);

    window.set_anchor(layer_shell::Edge::Left, true);
    window.set_anchor(layer_shell::Edge::Right, false);
    window.set_anchor(layer_shell::Edge::Top, true);
    window.set_anchor(layer_shell::Edge::Bottom, false);
    window.set_margin(layer_shell::Edge::Left, left);
    window.set_margin(layer_shell::Edge::Top, top);
}

#[cfg(not(feature = "layer-shell"))]
#[allow(dead_code)]
fn center_layer_shell(_window: &gtk::ApplicationWindow, _height: i32) {}

fn trigger_primary_action(
    launcher: &Launcher,
    state: &Rc<RefCell<UiState>>,
    list: &gtk::ListBox,
    entry: &gtk::Entry,
    window: &gtk::ApplicationWindow,
) {
    if let Some(row) = list.selected_row() {
        let idx = row.index().max(0) as usize;
        if let Some(item) = state.borrow().results.get(idx).cloned() {
            launcher.launch_app(&item.app);
            window.close();
            return;
        }
    }

    let q = entry.text().to_string();
    if launcher.web_search(&q) {
        window.close();
    }
}

fn refresh_results(
    launcher: &Launcher,
    entry: &gtk::Entry,
    list: &gtk::ListBox,
    scroller: &gtk::ScrolledWindow,
    status: &gtk::Label,
    container: &gtk::Box,
    window: &gtk::ApplicationWindow,
    state: &Rc<RefCell<UiState>>,
) {
    let query = entry.text().to_string();
    let trimmed = query.trim();
    let results = launcher.rank(trimmed, RESULT_LIMIT);

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    for result in &results {
        let row = gtk::ListBoxRow::new();
        row.add_css_class("seekx-row");
        let label = gtk::Label::new(Some(&result.app.name));
        label.set_xalign(0.0);
        label.add_css_class("seekx-label");
        row.set_child(Some(&label));
        list.append(&row);
    }

    status.set_text(&format!(
        "Installed: {} | Matches: {}",
        launcher.app_count(),
        results.len()
    ));

    if let Some(row) = list.row_at_index(0) {
        list.select_row(Some(&row));
    }

    let has_results = !results.is_empty();
    scroller.set_visible(has_results || !trimmed.is_empty());
    container.set_size_request(-1, WINDOW_HEIGHT - 24);
    window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    window.set_size_request(WINDOW_WIDTH, WINDOW_HEIGHT);
    center_layer_shell(window, WINDOW_HEIGHT);
    window.present();

    state.borrow_mut().results = results;
}

fn move_selection(list: &gtk::ListBox, state: &Rc<RefCell<UiState>>, delta: i32) {
    let total = state.borrow().results.len();
    if total == 0 {
        return;
    }

    let current = list.selected_row().map(|row| row.index()).unwrap_or(0);
    let next = (current + delta).clamp(0, total.saturating_sub(1) as i32);
    if let Some(row) = list.row_at_index(next) {
        list.select_row(Some(&row));
    }
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
window.seekx-window,
window.seekx-window > * {
  background: #000000;
}

*,
*:focus,
*:focus-visible,
*:selected {
  outline: none;
  box-shadow: none;
}

.seekx-root {
  background: #000000;
  border: 0.5px solid #ffffff;
  border-radius: 10px;
  padding: 16px;
}

entry.seekx-entry,
entry.seekx-entry text {
  background: transparent;
  color: #ffffff;
  border: none;
  border-radius: 0;
  font-size: 17px;
  box-shadow: none;
  outline: none;
}

entry.seekx-entry {
  min-height: 40px;
  padding: 0;
}

entry.seekx-entry:focus {
  outline: none;
  box-shadow: none;
  border: none;
}

entry.seekx-entry:focus-visible,
row.seekx-row:focus,
row.seekx-row:focus-visible,
list.seekx-list:focus,
list.seekx-list:focus-visible,
scrolledwindow.seekx-scroll:focus,
scrolledwindow.seekx-scroll:focus-visible {
  outline: none;
  box-shadow: none;
  border: none;
}

scrolledwindow.seekx-scroll,
scrolledwindow.seekx-scroll > viewport,
scrolledwindow.seekx-scroll > viewport > * {
  background: transparent;
  border: none;
  box-shadow: none;
}

list.seekx-list {
  background: transparent;
  border: none;
}

row.seekx-row {
  background: transparent;
  border: none;
  border-radius: 0;
  margin-top: 4px;
  margin-bottom: 4px;
  padding: 0;
}

row.seekx-row:selected {
  background: transparent;
  border: none;
}

label.seekx-label {
  color: #ffffff;
  font-size: 15px;
}

label.seekx-status {
  color: #ffffff;
  font-size: 12px;
}
",
    );

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
