//! 应用回调绑定与数据转换
//!
//! 将 Slint UI 事件绑定到解析器/导出器逻辑，
//! 并提供 StunnerReport → Slint 数据模型的转换函数。

use crate::{MainWindow, MetaItem, PositionItem};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use stunner_viewer::exporter;
use stunner_viewer::model::{MeasurementPosition, StunnerReport};
use stunner_viewer::parser::{FileParser, StunnerParser};

/// 绑定所有 UI 回调
pub fn bind_callbacks(app: &MainWindow, _parser: &StunnerParser) {
    let report: std::rc::Rc<std::cell::RefCell<Option<StunnerReport>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    bind_open_file(app, &report);
    bind_export_xlsx(app, &report);
}

/// 绑定"打开文件"回调
fn bind_open_file(
    app: &MainWindow,
    report: &std::rc::Rc<std::cell::RefCell<Option<StunnerReport>>>,
) {
    let weak: slint::Weak<MainWindow> = app.as_weak();
    let report = report.clone();

    app.on_open_file_clicked(move || {
        let Some(app) = weak.upgrade() else { return };

        let dialog = rfd::FileDialog::new()
            .set_title("打开 Stunner bin 文件")
            .add_filter("Bin 文件", &["bin"])
            .add_filter("所有文件", &["*"]);

        let Some(path) = dialog.pick_file() else { return };

        app.set_is_loading(true);
        app.set_status_message(SharedString::from(&format!(
            "正在解析: {}",
            path.display()
        )));

        let parser = StunnerParser::new();
        match parser.parse_file(&path) {
            Ok(parsed) => {
                update_ui_from_report(&app, &parsed);
                app.set_status_message(SharedString::from(&format!(
                    "已加载: {} ({} 条光谱)",
                    parsed.filename(),
                    parsed.spectra.len()
                )));
                *report.borrow_mut() = Some(parsed);
            }
            Err(e) => {
                app.set_status_message(SharedString::from(&format!("解析失败: {}", e)));
                eprintln!("解析失败: {}", e);
            }
        }

        app.set_is_loading(false);
    });
}

/// 绑定"导出 XLSX"回调
fn bind_export_xlsx(
    app: &MainWindow,
    report: &std::rc::Rc<std::cell::RefCell<Option<StunnerReport>>>,
) {
    let weak: slint::Weak<MainWindow> = app.as_weak();
    let report = report.clone();

    app.on_export_xlsx_clicked(move || {
        let Some(app) = weak.upgrade() else { return };
        let Some(ref parsed) = *report.borrow() else {
            app.set_status_message(SharedString::from("没有可导出的数据"));
            return;
        };

        let dialog = rfd::FileDialog::new()
            .set_title("导出 XLSX")
            .add_filter("Excel 文件", &["xlsx"])
            .set_file_name("Stunner实验数据.xlsx");

        let Some(path) = dialog.save_file() else { return };

        app.set_status_message(SharedString::from("正在导出..."));

        match exporter::export_xlsx(&[parsed.clone()], &path) {
            Ok(()) => {
                app.set_status_message(SharedString::from(&format!(
                    "导出成功: {}",
                    path.display()
                )));
            }
            Err(e) => {
                app.set_status_message(SharedString::from(&format!("导出失败: {}", e)));
                eprintln!("导出失败: {}", e);
            }
        }
    });
}

/// 将解析结果更新到 UI 各属性
fn update_ui_from_report(app: &MainWindow, report: &StunnerReport) {
    app.set_current_filename(SharedString::from(report.filename()));

    app.set_experiment_info(to_meta_items(&[
        ("实验名称", &report.experiment.name),
        ("样品类型", &report.experiment.sample_type),
        ("用户", &report.user_name),
        ("布局条码", &report.layout.barcode),
        ("芯片ID", &report.layout.chip_id_code),
        ("实验日期", &report.experiment.date),
    ]));

    app.set_instrument_info(to_meta_items(&[
        ("序列号", &report.instrument.serial_number),
        ("MAC地址", &report.instrument.mac_address),
        ("组装ID", &report.instrument.assembly_id),
        ("软件类型", &report.instrument.software_type),
        ("软件版本", &report.instrument.software_version),
        ("控制软件版本", &report.instrument.control_sw_version),
        ("固件版本", &report.instrument.firmware),
        ("SPM", &report.instrument.spm),
    ]));

    app.set_positions(to_position_items(&report.layout.positions));
    app.set_spectrum_path_commands(SharedString::from(&spectrum_to_svg_path(report)));
}

// === 数据转换 ===

/// 键值对 → Slint MetaItem 模型
fn to_meta_items(items: &[(&str, &Option<String>)]) -> ModelRc<MetaItem> {
    let vec: Vec<MetaItem> = items
        .iter()
        .map(|(key, val)| MetaItem {
            key: SharedString::from(*key),
            value: SharedString::from(val.as_deref().unwrap_or("-")),
        })
        .collect();
    ModelRc::new(VecModel::from(vec))
}

/// 测量位置 → Slint PositionItem 模型
fn to_position_items(positions: &[MeasurementPosition]) -> ModelRc<PositionItem> {
    let vec: Vec<PositionItem> = positions
        .iter()
        .map(|p| PositionItem {
            index: p.index as i32,
            name: SharedString::from(&p.name),
        })
        .collect();
    ModelRc::new(VecModel::from(vec))
}

/// 光谱数据 → SVG Path 命令（所有光谱叠加渲染）
///
/// 所有光谱共享同一坐标系：X 轴 190-790nm，Y 轴根据全局最大值缩放。
fn spectrum_to_svg_path(report: &StunnerReport) -> String {
    if report.spectra.is_empty() {
        return String::new();
    }

    // 计算全局 X/Y 范围
    let x_min = 190.0_f32;
    let x_max = 790.0_f32;
    let x_range = x_max - x_min;
    let mut global_y_max = f32::NEG_INFINITY;
    let mut global_y_min = f32::INFINITY;
    for spec in &report.spectra {
        global_y_max = global_y_max.max(spec.max_value());
        global_y_min = global_y_min.min(spec.min_value());
    }
    let y_range = if global_y_max > global_y_min {
        global_y_max - global_y_min
    } else {
        1.0
    };
    let y_margin = y_range * 0.2;
    let scale_x = 1000.0 / x_range;

    // 为每条光谱生成独立的 M/L 路径
    let mut cmds = String::new();
    for spec in &report.spectra {
        if spec.values.is_empty() {
            continue;
        }
        for (i, val) in spec.values.iter().enumerate() {
            let wl = spec.wavelengths.get(i).copied().unwrap_or(x_min + i as f32);
            let x = (wl - x_min) * scale_x;
            let y =
                200.0 + (global_y_max - val + y_margin) / (y_range + y_margin * 2.0) * 800.0;
            if i == 0 {
                cmds.push_str(&format!("M {:.1} {:.1}", x, y));
            } else {
                cmds.push_str(&format!(" L {:.1} {:.1}", x, y));
            }
        }
    }
    cmds
}
