#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod database;
mod windows;

use crate::database::connection::establish_connection;
use tokio::runtime::Runtime;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Registry};
use windows::main_window::MainWindow;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use tracing_subscriber::fmt::format::FmtSpan;

    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    let stdout_log = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::ACTIVE)
        .pretty();
    let subscriber = Registry::default().with(stdout_log);

    // tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    eframe::run_native(
        "Dofus farmer",
        Default::default(),
        Box::new(|cc| Box::new(MainWindow::new(cc, establish_connection()))),
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
