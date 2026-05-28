use anyhow::Result;

use archeo_gui::gui::app::ArcheoGuiApp;

fn main() -> Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "archeo-gui",
        options,
        Box::new(|_cc| Ok(Box::new(ArcheoGuiApp::new()))),
    )?;

    Ok(())
}
