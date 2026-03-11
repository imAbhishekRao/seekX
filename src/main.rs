mod application;
mod domain;
mod infrastructure;
mod settings;
mod ui;

fn main() {
    let apps = infrastructure::desktop::load_installed_apps();
    let launcher = application::Launcher::new(apps);
    ui::run(launcher);
}
