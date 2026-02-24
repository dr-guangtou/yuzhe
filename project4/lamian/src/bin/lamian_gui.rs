fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "LaMian GUI",
        options,
        Box::new(|creation_context| Box::new(lamian::gui::LamianGuiApp::new(creation_context))),
    )
}
