#![warn(clippy::all, rust_2018_idioms)]

#[cfg(not(target_arch = "wasm32"))]
mod app;
#[cfg(not(target_arch = "wasm32"))]
pub use app::SPSPlotApp;
#[cfg(not(target_arch = "wasm32"))]
mod nuclear_data;
#[cfg(not(target_arch = "wasm32"))]
mod excitation_fetchor;

#[cfg(target_arch = "wasm32")]
mod web_app;

#[cfg(target_arch = "wasm32")]
pub use web_app::SPSPlotApp;