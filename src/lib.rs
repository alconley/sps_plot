#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::SPSPlotApp;
mod nuclear_data_amdc_2016;
mod excitation_levels_nndc;