use eframe::egui;

pub struct CalibrationModal {}

pub enum CalibrationModalEffect {}

impl CalibrationModal {
    pub fn open(&mut self) {}

    pub fn show(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<CalibrationModalEffect> {
        None
    }
}
