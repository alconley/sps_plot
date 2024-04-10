use eframe::App;

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SPSPlotApp {
    window: bool,
}

impl SPSPlotApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, window: bool) -> Self {
        Self {
            window,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Web is not yet supported. Soon tho...");
    }
}

impl App for SPSPlotApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        
        if self.window {
            egui::Window::new("SPS Plot")
                .max_height(900.0)
                .show(ctx, |ui| {
                    self.ui(ui);
                });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.ui(ui);
            });
        }
    }
}
