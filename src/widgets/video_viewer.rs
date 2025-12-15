use eframe::egui::{self, Color32, ColorImage, RichText};
use mocap_for_one::{OpenCvCamera, mat_to_color_image};
use opencv::{
    core::MatTraitConst, core::Scalar, objdetect::draw_detected_markers,
};

pub enum VideoViewerEffect {
    OnClose,
    OnStartCalibration,
    OnCaptureFrame,
    OnStopCalibration,
}

pub struct VideoViewer {}

impl VideoViewer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        color_img: &ColorImage,
        on_calibration: bool,
    ) -> Option<VideoViewerEffect> {
        let mut ret = None;

        let total_width = ui.available_width();
        let image_width = total_width * 0.8;
        let config_width = total_width * 0.2;

        let available_height = ui.available_height();
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::Vec2::new(image_width, available_height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.centered_and_justified(|ui| {
                        let texture = ui.ctx().load_texture(
                            format!("cam_frame_"),
                            color_img.clone(),
                            Default::default(),
                        );

                        let img_size = color_img.size;
                        let available =
                            egui::Vec2::new(image_width, available_height);

                        let scale_x = available.x / img_size[0] as f32;
                        let scale_y = available.y / img_size[1] as f32;
                        let scale = scale_x.min(scale_y).min(1.0);

                        let display_size = egui::Vec2::new(
                            img_size[0] as f32 * scale,
                            img_size[1] as f32 * scale,
                        );

                        ui.image(egui::ImageSource::Texture(
                            egui::load::SizedTexture::new(
                                texture.id(),
                                display_size,
                            ),
                        ));
                    });
                },
            );

            ui.separator();

            ui.allocate_ui_with_layout(
                egui::Vec2::new(config_width, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.heading("Camera Config");
                    ui.separator();

                    if ui.button("Close").clicked() {
                        ret = Some(VideoViewerEffect::OnClose);
                    }

                    if ui
                        .button(
                            RichText::new("Start Calibration")
                                .color(Color32::GREEN),
                        )
                        .clicked()
                    {
                        ret = Some(VideoViewerEffect::OnStartCalibration);
                    }

                    if on_calibration {
                        if ui.button("Capture Frame").clicked() {
                            ret = Some(VideoViewerEffect::OnCaptureFrame);
                        }

                        if ui.button("Stop Calibration").clicked() {
                            ret = Some(VideoViewerEffect::OnStopCalibration);
                        }
                    }

                    ui.label("Camera Settings:");
                    ui.add(
                        egui::Slider::new(&mut 50.0, 0.0..=100.0)
                            .text("Brightness"),
                    );
                    ui.add(
                        egui::Slider::new(&mut 50.0, 0.0..=100.0)
                            .text("Contrast"),
                    );
                    ui.add(
                        egui::Slider::new(&mut 50.0, 0.0..=100.0)
                            .text("Saturation"),
                    );
                    ui.add(
                        egui::Slider::new(&mut 30.0, 1.0..=60.0).text("FPS"),
                    );

                    ui.separator();

                    ui.label("Display Settings:");
                    ui.checkbox(&mut false, "Show crosshair");
                    ui.checkbox(&mut false, "Show grid");
                    ui.checkbox(&mut true, "Auto-fit image");

                    ui.separator();

                    ui.label("Recording:");
                    if ui.button("Start Recording").clicked() {
                        // TODO: Implement recording
                    }
                    if ui.button("Take Screenshot").clicked() {
                        // TODO: Implement screenshot
                    }

                    ui.separator();

                    // ui.label(format!("Camera: {}", tab));
                    // if let Some(stream) = self.streams.get(tab) {
                    //     if let Some(img) = stream.latest_image() {
                    //         let img_size = img.size;
                    //         ui.label(format!(
                    //             "Resolution: {}x{}",
                    //             img_size[0], img_size[1]
                    //         ));
                    //     }
                    // }
                },
            );
        });

        ret
    }
}
