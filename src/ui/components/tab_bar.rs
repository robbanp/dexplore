use crate::models::Tab;
use eframe::egui;

#[derive(Debug)]
pub enum TabBarEvent {
    TabActivated(usize),
    TabClosed(usize),
}

pub struct TabBar;

impl TabBar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, tabs: &[Tab], active_tab: usize) -> Option<TabBarEvent> {
        let mut event = None;

        if !tabs.is_empty() {
            ui.horizontal(|ui| {
                for (i, tab) in tabs.iter().enumerate() {
                    let is_active = i == active_tab;
                    let tab_label = egui::RichText::new(&tab.title).strong();

                    if ui.selectable_label(is_active, tab_label).clicked() {
                        event = Some(TabBarEvent::TabActivated(i));
                    }

                    if ui.small_button("✖").clicked() {
                        event = Some(TabBarEvent::TabClosed(i));
                    }

                    ui.separator();
                }
            });

            ui.separator();
        }

        event
    }
}
