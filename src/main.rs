use chrono::format;
use eframe::egui;
use eframe::epaint::image;
use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, thread, time::Duration};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

mod widgets;
use widgets::{TCamBuffers, UnityCameraModal, VideoCaptureModal};

struct App {
    tcam_buffers: TCamBuffers,
    tree: egui_dock::DockState<String>,
    video_modal: VideoCaptureModal,
    unity_modal: UnityCameraModal,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // show camera open modal if requested
        self.video_modal.show(ctx, &mut self.tcam_buffers, &mut self.tree);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button("Add Video Capture").clicked() {
                    self.video_modal.open();
                }

                if ui.button("Add Unity Camera").clicked() {
                    self.unity_modal.open();
                }
            });
        });

        // show unity file dialog if active
        self.unity_modal.show(ctx, &mut self.tcam_buffers, &mut self.tree);

        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tcam_buffers);

        thread::sleep(Duration::from_millis(1000 / 60));
        ctx.request_repaint();
    }
}

impl App {
    fn new() -> Self {
        let tree = egui_dock::DockState::new(vec![]);

        Self {
            tcam_buffers: TCamBuffers {
                tcams: BTreeMap::new(),
            },
            tree,
            video_modal: VideoCaptureModal::new(),
            unity_modal: UnityCameraModal::new(),
        }
    }

    fn render_cam(
        &mut self,
        ui: &mut egui::Ui,
    ) -> opencv::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MocapForOne",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
