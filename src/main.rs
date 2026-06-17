//! Stunner Viewer 应用入口
//!
//! Slint UI 类型在此生成，回调绑定在 app.rs。

mod app;

use stunner_viewer::parser::StunnerParser;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let parser = StunnerParser::new();

    app::bind_callbacks(&app, &parser);

    app.run()
}
