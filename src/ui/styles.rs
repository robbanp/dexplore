use eframe::egui;

/// Setup monospace font styles for better data display
pub fn setup_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(11.0, egui::FontFamily::Monospace)
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(11.0, egui::FontFamily::Monospace)
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(14.0, egui::FontFamily::Monospace)
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(9.0, egui::FontFamily::Monospace)
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(11.0, egui::FontFamily::Monospace)
    );

    ctx.set_style(style);
}
