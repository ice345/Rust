use chess_gui::ChessApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Chess Game"),
        ..Default::default()
    };

    eframe::run_native(
        "Chess Game",
        options,
        Box::new(|_cc| Ok(Box::new(ChessApp::new()))),
    )
}
