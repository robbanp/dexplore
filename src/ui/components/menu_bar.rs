use eframe::egui;

#[derive(Debug)]
pub enum MenuBarEvent {
    ShowSettings,
    Quit,
    ToggleQueryPanel,
    Refresh,
}

pub struct MenuBar;

impl MenuBar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, connection_status: &str) -> Option<MenuBarEvent> {
        let mut event = None;

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Settings...").clicked() {
                    event = Some(MenuBarEvent::ShowSettings);
                    ui.close_menu();
                }
                if ui.button("Quit").clicked() {
                    event = Some(MenuBarEvent::Quit);
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Show Query Panel").clicked() {
                    event = Some(MenuBarEvent::ToggleQueryPanel);
                }
            });

            ui.separator();

            if ui.button("🔄 Refresh").clicked() {
                event = Some(MenuBarEvent::Refresh);
            }

            if ui.button("📝 Query").clicked() {
                event = Some(MenuBarEvent::ToggleQueryPanel);
            }

            ui.separator();
            ui.label(connection_status);
        });

        event
    }
}
