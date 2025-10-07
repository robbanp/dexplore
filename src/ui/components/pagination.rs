use eframe::egui;

#[derive(Debug)]
pub enum PaginationEvent {
    Reload,
    PageSizeChanged(usize),
    PageChanged(usize),
}

pub struct PaginationControls;

impl PaginationControls {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        current_page: usize,
        page_size: usize,
        total_rows: usize,
    ) -> Option<PaginationEvent> {
        let mut event = None;

        let total_pages = total_rows.div_ceil(page_size);
        let start_row = current_page * page_size;
        let end_row = (start_row + page_size).min(total_rows);

        ui.horizontal(|ui| {
            if ui.button("🔄 Reload").clicked() {
                event = Some(PaginationEvent::Reload);
            }

            ui.separator();

            ui.label("Rows per page:");

            for size in [50, 100, 500, 1000, 5000] {
                let is_selected = page_size == size;
                if ui.selectable_label(is_selected, format!("{}", size)).clicked() {
                    event = Some(PaginationEvent::PageSizeChanged(size));
                }
            }

            ui.separator();

            if ui.button("◀ Previous").clicked() && current_page > 0 {
                event = Some(PaginationEvent::PageChanged(current_page - 1));
            }

            ui.label(format!(
                "Page {} of {} ({}-{} of {} rows)",
                current_page + 1,
                total_pages.max(1),
                start_row + 1,
                end_row,
                total_rows
            ));

            if ui.button("Next ▶").clicked() && current_page + 1 < total_pages {
                event = Some(PaginationEvent::PageChanged(current_page + 1));
            }
        });

        ui.separator();

        event
    }
}
