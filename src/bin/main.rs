#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod data_loader;
mod database;
mod windows;

use crate::database::connection::establish_pooled_connection;
use eframe::NativeOptions;
use tokio::runtime::Runtime;
use tracing::level_filters::LevelFilter;
use tracing::trace_span;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use windows::main_window::MainWindow;

fn main() {
    let stdout_log = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::ACTIVE)
        .pretty();
    let _subscriber = Registry::default()
        .with(stdout_log)
        .with(LevelFilter::from_level(Level::INFO));

    tracing::subscriber::set_global_default(_subscriber).expect("Unable to set global subscriber");

    start().unwrap();
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn start() -> eframe::Result<()> {
    use egui::{FontId, Style, Visuals};
    use tracing::{event, Level};

    let span = trace_span!("starting main");
    let _guard = span.enter();

    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    event!(Level::TRACE, "establishing  pooled connection");
    let pool = establish_pooled_connection();

    let options = NativeOptions::default();

    eframe::run_native(
        "Dofus farmer",
        options,
        Box::new(|creation_context| {
            let style = Style {
                visuals: Visuals::dark(),
                override_font_id: Some(FontId::proportional(17f32)),
                ..Style::default()
            };

            creation_context.egui_ctx.set_style(style);
            Ok(Box::new(MainWindow::new(creation_context, pool)))
        }),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
