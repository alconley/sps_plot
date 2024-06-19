use log::info;

use eframe::egui::{self, Color32, RichText, Stroke};
use eframe::App;
use egui_plot::{Bar, BarChart, Legend, Orientation, Plot, PlotBounds, VLine};

use std::collections::HashMap;
use std::f64::consts::PI;

use super::excitation_fetchor::ExcitationFetcher;
use super::nuclear_data::{MassMap, NuclearData};

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

    pub rho_values: Vec<(f64, f64)>,
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
    is_loading: bool,
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
            is_loading: false,
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
            is_loading: false,
            window,
        };

        if let Some(storage) = cc.storage {
            app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        app
    }

    fn sps_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("SE-SPS Settings")
                .color(Color32::LIGHT_BLUE)
                .size(18.0),
        );

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
        });
    }

    fn reaction_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Reactions")
                    .color(Color32::LIGHT_BLUE)
                    .size(18.0),
            );

            ui.separator();

            if ui.button("Calculate").clicked() {
                self.calculate_rho_for_all_reactions();
            }

            ui.separator();

            if ui.button("+").clicked() {
                self.reactions.push(Reaction::default());
            }
        });

        ui.separator();

        let mut index_to_remove: Option<usize> = None;

        for (index, reaction) in self.reactions.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // let reaction = &mut self.reactions;
                ui.label(format!("Reaction {}", index));

                ui.separator();

                if ui.button("-").clicked() {
                    index_to_remove = Some(index);
                }

                ui.separator();

                ui.label("Target: ");
                ui.add(egui::DragValue::new(&mut reaction.target_z).prefix("Z: "));
                ui.add(egui::DragValue::new(&mut reaction.target_a).prefix("A: "));

                ui.separator();

                ui.label("Projectile: ");
                ui.add(egui::DragValue::new(&mut reaction.projectile_z).prefix("Z: "));
                ui.add(egui::DragValue::new(&mut reaction.projectile_a).prefix("A: "));

                ui.separator();

                ui.label("Ejectile: ");
                ui.add(egui::DragValue::new(&mut reaction.ejectile_z).prefix("Z: "));
                ui.add(egui::DragValue::new(&mut reaction.ejectile_a).prefix("A: "));

                ui.separator();

                ui.label(reaction.reaction_identifier.to_string());

                if ui.button("Get Reaction").clicked() {
                    Self::populate_reaction_data(reaction);
                    Self::fetch_excitation_levels(reaction);
                }

                ui.separator();
            });
        }

        if let Some(index) = index_to_remove {
            self.reactions.remove(index);
        }
    }

    fn populate_reaction_data(reaction: &mut Reaction) {
        let mass_map = match MassMap::new() {
            Ok(map) => map,
            Err(e) => {
                log::error!("Failed to initialize MassMap: {}", e);
                MassMap::default()
            }
        };

        reaction.target_data = mass_map
            .get_data(&(reaction.target_z as u32), &(reaction.target_a as u32))
            .cloned();
        reaction.projectile_data = mass_map
            .get_data(
                &(reaction.projectile_z as u32),
                &(reaction.projectile_a as u32),
            )
            .cloned();
        reaction.ejectile_data = mass_map
            .get_data(&(reaction.ejectile_z as u32), &(reaction.ejectile_a as u32))
            .cloned();

        reaction.resid_z = reaction.target_z + reaction.projectile_z - reaction.ejectile_z;
        reaction.resid_a = reaction.target_a + reaction.projectile_a - reaction.ejectile_a;

        reaction.resid_data = mass_map
            .get_data(&(reaction.resid_z as u32), &(reaction.resid_a as u32))
            .cloned();

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

        // Using an async block, note that this requires an executor to run the block synchronously
        let fetcher = ExcitationFetcher::new();
        fetcher.fetch_excitation_levels(isotope);

        let levels_lock = fetcher.excitation_levels.lock().unwrap();
        let error_lock = fetcher.error_message.lock().unwrap();

        if let Some(levels) = &*levels_lock {
            info!("Fetched excitation levels: {:?}", levels);
            reaction.excitation_levels = levels.clone();
            // return levels.clone();
        } else if let Some(error) = &*error_lock {
            log::error!("Error fetching excitation levels: {}", error);
        }
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

        for excitation in &reaction.excitation_levels {
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
            let ke2 = term1 - (term1 * term1 + term2).sqrt();

            let ejectile_energy = if ke1 > 0.0 { ke1 * ke1 } else { ke2 * ke2 };

            // convert ejectile ke to rho
            let p = (ejectile_energy * (ejectile_energy + 2.0 * ejectile.mass)).sqrt();
            let qbrho = p / QBRHO2P;
            let rho = qbrho / (magnetic_field * ejectile.z as f64);
            info!("Excitation: {}, rho: {}", excitation, rho);

            reaction.rho_values.push((*excitation, rho));
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

    fn rho_to_barchart(reaction: &mut Reaction, y_value: f64, color: Color32) -> BarChart {
        let mut bars = Vec::new();
        for (excitation, rho) in &reaction.rho_values {
            let bar = Bar {
                orientation: Orientation::Vertical,
                argument: *rho,
                value: 0.50,
                bar_width: 0.01,
                fill: color,
                stroke: Stroke::new(1.0, color),
                name: format!("E = {:.3} MeV\nrho = {:.3}\n", *excitation, *rho),
                base_offset: Some(y_value),
            };

            bars.push(bar);
        }

        BarChart::new(bars)
            .name(reaction.reaction_identifier.clone())
            .color(color)
            .highlight(true)
    }

    fn plot(&mut self, ui: &mut egui::Ui) {
        let plot = Plot::new("SPS Plot")
            .show_y(false)
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .legend(Legend::default());

        let colors = [
            Color32::LIGHT_BLUE,
            Color32::LIGHT_GREEN,
            Color32::LIGHT_RED,
            Color32::LIGHT_YELLOW,
            Color32::LIGHT_GRAY,
            Color32::BLUE,
            Color32::GREEN,
            Color32::RED,
            Color32::YELLOW,
            Color32::GRAY,
        ];

        plot.show(ui, |plot_ui| {
            // plots the rho values
            plot_ui.vline(VLine::new(self.rho_min).color(Color32::RED));
            plot_ui.vline(VLine::new(self.rho_max).color(Color32::RED));

            for (index, reaction) in self.reactions.iter_mut().enumerate() {
                let color = colors[index % colors.len()];
                let y_value = index as f64 + 0.25;
                let barchart = Self::rho_to_barchart(reaction, y_value, color);
                plot_ui.bar_chart(barchart);
            }

            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                (self.rho_min - 5.0, -1.0).into(),
                (self.rho_max + 5.0, self.reactions.len() as f64 + 1.0).into(),
            ));
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("sps_plot_top_panel").show_inside(ui, |ui| {
            self.sps_settings_ui(ui);
        });

        egui::TopBottomPanel::bottom("sps_plot_bottom_panel").show_inside(ui, |ui| {
            self.reaction_ui(ui);
        });

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
                ui.label("Reaction Data");
                for (reaction, data) in &self.reaction_data {
                    ui.label(format!("{}: {:?}", reaction, data));
                }
                self.ui(ui);
            });
        }
    }
}
