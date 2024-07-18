use log::info;

use eframe::egui::{self, Color32, Stroke};
use eframe::App;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart, Legend, Orientation, Plot, PlotBounds, VLine};

use std::collections::HashMap;
use std::f64::consts::PI;

use super::excitation_levels_nndc::ExcitationLevels;
use super::nuclear_data_amdc_2016::NuclearData;

const C: f64 = 299792458.0; // Speed of light in m/s
const QBRHO2P: f64 = 1.0E-9 * C; // Converts qbrho to momentum (p) (kG*cm -> MeV/c)

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct Reaction {
    pub target_z: i32,
    pub target_a: i32,
    pub target_data: Option<NuclearData>,

    pub projectile_z: i32,
    pub projectile_a: i32,
    pub projectile_data: Option<NuclearData>,

    pub ejectile_z: i32,
    pub ejectile_a: i32,
    pub ejectile_data: Option<NuclearData>,

    pub resid_z: i32,
    pub resid_a: i32,
    pub resid_data: Option<NuclearData>,

    pub reaction_identifier: String,

    pub excitation_levels: Vec<f64>,
    pub add_excitation_level: f64,
    pub additional_excitation_levels: Vec<f64>,

    pub rho_values: Vec<(f64, f64)>,

    pub color: Color32,
}

impl Reaction {
    pub fn new(color: egui::Color32) -> Self {
        Reaction {
            color,
            ..Default::default()
        }
    }

    pub fn excitation_levels_ui(&mut self, ui: &mut egui::Ui, index: usize) {
        egui::ScrollArea::vertical()
            .id_source(format!("Reaction {} Scroll Area", index))
            .show(ui, |ui| {
                // ui.vertical(|ui| {

                ui.label(self.reaction_identifier.clone());
                ui.horizontal(|ui| {
                    ui.label("Color: ");
                    ui.color_edit_button_srgba(&mut self.color);
                });
                ui.label("Excitation Levels");
                ui.separator();

                if self.excitation_levels.is_empty() {
                    ui.label("None");
                }

                let mut to_remove_level: Option<usize> = None;
                for (index, level) in self.excitation_levels.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {:.3} MeV", index, level));
                        if ui.button("-").clicked() {
                            to_remove_level = Some(index);
                        }
                    });
                }

                if let Some(index) = to_remove_level {
                    self.excitation_levels.remove(index);
                }

                ui.separator();

                ui.label("Additional Levels");

                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.add_excitation_level)
                            .prefix("Custom: ")
                            .suffix(" MeV")
                            .speed(0.1)
                            .clamp_range(0.0..=f64::MAX),
                    );
                    if ui.button("+").clicked() {
                        self.additional_excitation_levels
                            .push(self.add_excitation_level);
                        log::info!("Added new excitation level: {}", self.add_excitation_level);
                    }
                });

                let mut to_remove: Option<usize> = None;
                if !self.additional_excitation_levels.is_empty() {
                    for (index, level) in self.additional_excitation_levels.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Energy: {} MeV", level));
                            if ui.button("-").clicked() {
                                to_remove = Some(index);
                            }
                        });
                    }

                    if let Some(index) = to_remove {
                        self.additional_excitation_levels.remove(index);
                    }
                }
                // });
            });
    }

    pub fn settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Target: ");
        ui.add(egui::DragValue::new(&mut self.target_z).prefix("Z: "));
        ui.add(egui::DragValue::new(&mut self.target_a).prefix("A: "));

        ui.separator();

        ui.label("Projectile: ");
        ui.add(egui::DragValue::new(&mut self.projectile_z).prefix("Z: "));
        ui.add(egui::DragValue::new(&mut self.projectile_a).prefix("A: "));

        ui.separator();

        ui.label("Ejectile: ");
        ui.add(egui::DragValue::new(&mut self.ejectile_z).prefix("Z: "));
        ui.add(egui::DragValue::new(&mut self.ejectile_a).prefix("A: "));

        ui.separator();

        ui.label(self.reaction_identifier.to_string());

        if ui.button("Get Reaction").clicked() {
            Self::populate_reaction_data(self);
            Self::fetch_excitation_levels(self);
        }
    }

    pub fn draw(&self, plot_ui: &mut egui_plot::PlotUi, y_offset: f64) {
        let color = self.color;

        let mut bars = Vec::new();
        for (excitation, rho) in &self.rho_values {
            let bar = Bar {
                orientation: Orientation::Vertical,
                argument: *rho,
                value: 0.50,
                bar_width: 0.01,
                fill: color,
                stroke: Stroke::new(1.0, color),
                name: format!("E = {:.3} MeV\nrho = {:.3}\n", *excitation, *rho),
                base_offset: Some(y_offset),
            };

            bars.push(bar);
        }

        let barchart = BarChart::new(bars)
            .name(self.reaction_identifier.clone())
            .color(color)
            .highlight(true);

        plot_ui.bar_chart(barchart);
    }

    fn populate_reaction_data(reaction: &mut Reaction) {
        reaction.resid_z = reaction.target_z + reaction.projectile_z - reaction.ejectile_z;
        reaction.resid_a = reaction.target_a + reaction.projectile_a - reaction.ejectile_a;

        reaction.target_data =
            NuclearData::get_data(reaction.target_z as u32, reaction.target_a as u32);
        reaction.projectile_data =
            NuclearData::get_data(reaction.projectile_z as u32, reaction.projectile_a as u32);
        reaction.ejectile_data =
            NuclearData::get_data(reaction.ejectile_z as u32, reaction.ejectile_a as u32);
        reaction.resid_data =
            NuclearData::get_data(reaction.resid_z as u32, reaction.resid_a as u32);

        reaction.reaction_identifier = format!(
            "{}({},{}){}",
            reaction
                .target_data
                .as_ref()
                .map_or("None", |data| &data.isotope),
            reaction
                .projectile_data
                .as_ref()
                .map_or("None", |data| &data.isotope),
            reaction
                .ejectile_data
                .as_ref()
                .map_or("None", |data| &data.isotope),
            reaction
                .resid_data
                .as_ref()
                .map_or("None", |data| &data.isotope)
        );

        info!("Reaction: {:?}", reaction);
    }

    fn fetch_excitation_levels(reaction: &mut Reaction) {
        let isotope = reaction
            .resid_data
            .as_ref()
            .map_or("None", |data| &data.isotope);
        if isotope == "None" {
            log::error!(
                "No isotope found for reaction: {}",
                reaction.reaction_identifier
            );
        }

        let excitation_levels = ExcitationLevels::new();

        if let Some(levels) = excitation_levels.get(isotope) {
            log::info!("Excitation levels for {}: {:?}", isotope, levels);
            reaction.excitation_levels = levels;

            log::info!(
                "Excitation levels for {}: {:?}",
                isotope,
                reaction.excitation_levels.clone()
            );
        } else {
            log::error!("No excitation levels found for {}.", isotope);
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SPSPlotApp {
    sps_angle: f64,
    beam_energy: f64,
    magnetic_field: f64,
    rho_min: f64,
    rho_max: f64,
    reactions: Vec<Reaction>,
    reaction_data: HashMap<String, Vec<(f64, f64)>>,
    side_panel: bool,
    window: bool,
}

impl Default for SPSPlotApp {
    fn default() -> Self {
        Self {
            sps_angle: 35.0,
            beam_energy: 16.0,
            magnetic_field: 8.7,
            rho_min: 69.0,
            rho_max: 87.0,
            reactions: Vec::new(),
            reaction_data: HashMap::new(),
            side_panel: false,
            window: false,
        }
    }
}

impl SPSPlotApp {
    pub fn new(cc: &eframe::CreationContext<'_>, window: bool) -> Self {
        let mut app = Self {
            sps_angle: 35.0,     // degree
            beam_energy: 16.0,   // MeV
            magnetic_field: 8.7, // kG
            rho_min: 69.0,
            rho_max: 87.0,
            reactions: Vec::new(),
            reaction_data: HashMap::new(),
            side_panel: false,
            window,
        };

        if let Some(storage) = cc.storage {
            app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        app
    }

    fn sps_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            egui::global_dark_light_mode_switch(ui);

            ui.heading("SE-SPS Settings");
        });

        ui.horizontal(|ui| {
            ui.label("SPS Angle: ")
                .on_hover_text("SE-SPS's angle currently limited to 60°");
            ui.add(
                egui::DragValue::new(&mut self.sps_angle)
                    .suffix("°")
                    .clamp_range(0.0..=60.0),
            );

            ui.label("Beam Energy: ");
            ui.add(
                egui::DragValue::new(&mut self.beam_energy)
                    .suffix(" MeV")
                    .clamp_range(0.0..=f64::MAX),
            );

            ui.label("Magnetic Field: ");
            ui.add(
                egui::DragValue::new(&mut self.magnetic_field)
                    .suffix(" kG")
                    .clamp_range(0.0..=17.0),
            );

            ui.label("Rho Min: ")
                .on_hover_text("SE-SPS Rho Min is usually 69.0");
            ui.add(
                egui::DragValue::new(&mut self.rho_min)
                    .suffix(" cm")
                    .clamp_range(0.0..=f64::MAX),
            );

            ui.label("Rho Max: ")
                .on_hover_text("SE-SPS Rho Max is usually 87.0");
            ui.add(
                egui::DragValue::new(&mut self.rho_max)
                    .suffix(" cm")
                    .clamp_range(0.0..=f64::MAX),
            );

            ui.separator();

            if ui.button("Calculate").clicked() {
                self.calculate_rho_for_all_reactions();
            }

            ui.separator();

            ui.checkbox(&mut self.side_panel, "Show Exciation Levels");
        });
    }

    fn reactions_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Reactions");

            ui.separator();

            if ui.button("Calculate").clicked() {
                self.calculate_rho_for_all_reactions();
            }

            ui.separator();

            if ui.button("+").clicked() {
                let colors = [
                    Color32::from_rgb(120, 47, 64), // go noles!
                    Color32::from_rgb(206, 184, 136),
                    Color32::BLUE,
                    Color32::GREEN,
                    Color32::YELLOW,
                    Color32::BROWN,
                    Color32::DARK_RED,
                    Color32::RED,
                    Color32::LIGHT_RED,
                    Color32::LIGHT_YELLOW,
                    Color32::KHAKI,
                    Color32::DARK_GREEN,
                    Color32::LIGHT_GREEN,
                    Color32::DARK_BLUE,
                    Color32::LIGHT_BLUE,
                ];

                // change the default color to be random
                let index = self.reactions.len();
                let color = colors[index % colors.len()];

                self.reactions.push(Reaction::new(color));
            }
        });

        egui::ScrollArea::both().show(ui, |ui| {
            ui.separator();

            let mut index_to_remove: Option<usize> = None;

            for (index, reaction) in self.reactions.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("Reaction {}", index));

                    ui.separator();

                    if ui.button("-").clicked() {
                        index_to_remove = Some(index);
                    }

                    reaction.settings_ui(ui);
                });
            }

            if let Some(index) = index_to_remove {
                self.reactions.remove(index);
            }
        });
    }

    fn excitation_level_to_rho(
        reaction: &mut Reaction,
        beam_energy: f64,
        magnetic_field: f64,
        sps_angle: f64,
    ) {
        reaction.rho_values.clear();

        let target = reaction.target_data.as_ref().unwrap();
        let projectile = reaction.projectile_data.as_ref().unwrap();
        let ejectile = reaction.ejectile_data.as_ref().unwrap();
        let resid = reaction.resid_data.as_ref().unwrap();

        let reaction_identifier = format!(
            "{}({},{}){}",
            target.isotope, projectile.isotope, ejectile.isotope, resid.isotope
        );
        info!("Reaction: {}", reaction_identifier);

        let q_value = target.mass + projectile.mass - ejectile.mass - resid.mass;

        let mut levels = reaction.excitation_levels.clone();
        for level in reaction.additional_excitation_levels.iter() {
            levels.push(*level);
        }

        log::info!("Excitation levels: {:?}", levels);

        for excitation in levels {
            // for excitation in &reaction.excitation_levels {

            let reaction_q_value = q_value - excitation;
            // let beam_reaction_energy = self.beam_energy; // could put energy loss through target here
            let beam_reaction_energy = beam_energy; // could put energy loss through target here

            let _threshold = -reaction_q_value * (ejectile.mass + resid.mass)
                / (ejectile.mass + resid.mass - projectile.mass);
            let term1 = (projectile.mass * ejectile.mass * beam_reaction_energy).sqrt()
                / (ejectile.mass + resid.mass)
                * (sps_angle * PI / 180.0).cos();
            let term2 = (beam_reaction_energy * (resid.mass - projectile.mass)
                + resid.mass * reaction_q_value)
                / (ejectile.mass + resid.mass);

            let ke1 = term1 + (term1 * term1 + term2).sqrt();
            let ke2 = term1 + (term1 * term1 + term2).sqrt();

            let ejectile_energy = if ke1 > 0.0 { ke1 * ke1 } else { ke2 * ke2 };

            // convert ejectile ke to rho
            let p = (ejectile_energy * (ejectile_energy + 2.0 * ejectile.mass)).sqrt();
            let qbrho = p / QBRHO2P;
            let rho = qbrho / (magnetic_field * ejectile.z as f64);
            info!("Excitation: {}, rho: {}", excitation, rho);

            reaction.rho_values.push((excitation, rho));
        }
    }

    fn calculate_rho_for_all_reactions(&mut self) {
        for reaction in &mut self.reactions {
            Self::excitation_level_to_rho(
                reaction,
                self.beam_energy,
                self.magnetic_field,
                self.sps_angle,
            );
        }
    }

    fn excitation_levels_side_ui(&mut self, ui: &mut egui::Ui) {
        let height = ui.available_height();
        TableBuilder::new(ui)
            .columns(Column::auto().resizable(true), self.reactions.len())
            .body(|mut body| {
                body.row(height, |mut row| {
                    for (index, reaction) in &mut self.reactions.iter_mut().enumerate() {
                        row.col(|ui| {
                            reaction.excitation_levels_ui(ui, index);
                        });
                    }
                });
            });
    }

    fn plot(&mut self, ui: &mut egui::Ui) {
        let plot = Plot::new("SPS Plot")
            .show_y(false)
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .legend(Legend::default());

        plot.show(ui, |plot_ui| {
            // plots the rho values
            plot_ui.vline(VLine::new(self.rho_min).color(Color32::RED));
            plot_ui.vline(VLine::new(self.rho_max).color(Color32::RED));

            for (index, reaction) in self.reactions.iter_mut().enumerate() {
                let y_value = index as f64 + 0.25;
                reaction.draw(plot_ui, y_value);
            }

            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                (self.rho_min - 5.0, -1.0).into(),
                (self.rho_max + 5.0, self.reactions.len() as f64 + 1.0).into(),
            ));
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("sps_plot_top_panel").show_inside(ui, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                self.sps_settings_ui(ui);
            });
        });

        egui::TopBottomPanel::bottom("sps_plot_bottom_panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                self.reactions_ui(ui);
            });

        egui::SidePanel::left("sps_plot_side_panel").show_animated_inside(
            ui,
            self.side_panel,
            |ui| {
                self.excitation_levels_side_ui(ui);
            },
        );

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.plot(ui);
        });
    }
}

impl App for SPSPlotApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.window {
            egui::Window::new("SPS Plot")
                .max_height(900.0)
                .show(ctx, |ui| {
                    self.ui(ui);
                });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                for (reaction, data) in &self.reaction_data {
                    ui.label(format!("{}: {:?}", reaction, data));
                }
                self.ui(ui);
            });
        }
    }
}
