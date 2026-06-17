//! XLSX 导出模块
//!
//! 将 StunnerReport 导出为格式化的 Excel 文件，
//! 参考 export_stunner_xlsx.py 的样式和布局。

use crate::error::AppError;
use crate::model::StunnerReport;
use rust_xlsxwriter::{Format, FormatAlign, Workbook, XlsxError};
use std::path::Path;

/// 将多个报告导出为单个 XLSX 文件
///
/// 每个报告生成一个独立的工作表，包含：
/// - 实验信息
/// - 仪器信息
/// - 测量位置
/// - 吸光度阈值/过渡数据
/// - 光谱数据
pub fn export_xlsx(reports: &[StunnerReport], path: &Path) -> Result<(), AppError> {
    let mut workbook = Workbook::new();

    // 样式定义
    let title_fmt = title_format();
    let section_fmt = section_format();
    let header_fmt = header_format();
    let key_fmt = key_format();
    let value_fmt = value_format();
    let alt_key_fmt = alt_key_format();
    let alt_value_fmt = alt_value_format();

    for report in reports {
        let sheet_name = sheet_name(report);
        let worksheet = workbook
            .add_worksheet()
            .set_name(&sheet_name)
            .map_err(xlsx_err)?;

        // 设置列宽
        worksheet.set_column_width(0, 25).map_err(xlsx_err)?;
        worksheet.set_column_width(1, 35).map_err(xlsx_err)?;
        worksheet.set_column_width(2, 15).map_err(xlsx_err)?;
        worksheet.set_column_width(3, 15).map_err(xlsx_err)?;
        worksheet.set_column_width(4, 15).map_err(xlsx_err)?;

        let mut row: u32 = 0;

        // === 标题行 ===
        let title = format!("Stunner 实验数据: {}", report.filename());
        worksheet
            .merge_range(row, 0, row, 4, &title, &title_fmt)
            .map_err(xlsx_err)?;
        row += 2;

        // === 实验信息 ===
        row = write_section_header(worksheet, row, "📋 实验信息", &section_fmt);
        let exp_info: Vec<(&str, String)> = vec![
            ("实验名称", opt_str(&report.experiment.name)),
            ("样品类型", opt_str(&report.experiment.sample_type)),
            ("用户", opt_str(&report.user_name)),
            ("布局条码", opt_str(&report.layout.barcode)),
            ("芯片ID", opt_str(&report.layout.chip_id_code)),
        ];
        row = write_kv_rows(worksheet, row, &exp_info, &key_fmt, &value_fmt, &alt_key_fmt, &alt_value_fmt);
        row += 1;

        // === 仪器信息 ===
        row = write_section_header(worksheet, row, "🔬 仪器信息", &section_fmt);
        let inst_info: Vec<(&str, String)> = vec![
            ("序列号", opt_str(&report.instrument.serial_number)),
            ("MAC地址", opt_str(&report.instrument.mac_address)),
            ("组装ID", opt_str(&report.instrument.assembly_id)),
            ("软件类型", opt_str(&report.instrument.software_type)),
            ("软件版本", opt_str(&report.instrument.software_version)),
            ("控制软件版本", opt_str(&report.instrument.control_sw_version)),
            ("固件版本", opt_str(&report.instrument.firmware)),
            ("SPM", opt_str(&report.instrument.spm)),
        ];
        row = write_kv_rows(worksheet, row, &inst_info, &key_fmt, &value_fmt, &alt_key_fmt, &alt_value_fmt);
        row += 1;

        // === 测量位置 ===
        if !report.layout.positions.is_empty() {
            row = write_section_header(worksheet, row, "📍 测量位置", &section_fmt);
            // 表头
            write_table_header(worksheet, row, &["序号", "位置名称"], &header_fmt);
            row += 1;
            for (i, pos) in report.layout.positions.iter().enumerate() {
                let kf = if i % 2 == 0 { &key_fmt } else { &alt_key_fmt };
                let vf = if i % 2 == 0 { &value_fmt } else { &alt_value_fmt };
                worksheet
                    .write_with_format(row, 0, pos.index as i32, kf)
                    .map_err(xlsx_err)?;
                worksheet
                    .write_with_format(row, 1, &pos.name, vf)
                    .map_err(xlsx_err)?;
                row += 1;
            }
            row += 1;
        }

        // === 吸光度阈值 ===
        if !report.absorbance.thresholds.is_empty() {
            row = write_section_header(
                worksheet,
                row,
                "📊 吸光度阈值 (Absorbance Threshold)",
                &section_fmt,
            );
            write_table_header(worksheet, row, &["索引", "吸光度值"], &header_fmt);
            row += 1;
            for (i, val) in report.absorbance.thresholds.iter().enumerate() {
                let fmt = if i % 2 == 0 { &value_fmt } else { &alt_value_fmt };
                worksheet
                    .write_with_format(row, 0, i as i32, fmt)
                    .map_err(xlsx_err)?;
                worksheet
                    .write_with_format(row, 1, *val as f64, fmt)
                    .map_err(xlsx_err)?;
                row += 1;
            }
            row += 1;
        }

        // === 吸光度过渡 ===
        if !report.absorbance.transitions.is_empty() {
            row = write_section_header(
                worksheet,
                row,
                "📊 吸光度转换 (Absorbance Transition)",
                &section_fmt,
            );
            write_table_header(worksheet, row, &["索引", "转换值"], &header_fmt);
            row += 1;
            for (i, val) in report.absorbance.transitions.iter().enumerate() {
                let fmt = if i % 2 == 0 { &value_fmt } else { &alt_value_fmt };
                worksheet
                    .write_with_format(row, 0, i as i32, fmt)
                    .map_err(xlsx_err)?;
                worksheet
                    .write_with_format(row, 1, *val as f64, fmt)
                    .map_err(xlsx_err)?;
                row += 1;
            }
            row += 1;
        }

        // === 光谱数据 ===
        if !report.spectra.is_empty() {
            let total = report.spectra.len();
            row = write_section_header(
                worksheet,
                row,
                &format!("📈 光谱数据 (共 {} 条光谱)", total),
                &section_fmt,
            );
            for (spec_idx, spec) in report.spectra.iter().enumerate() {
                let label = format!(
                    "光谱 #{}: 位置 {} ({} 点)",
                    spec_idx + 1,
                    spec.position_index,
                    spec.values.len()
                );
                row = write_section_header(worksheet, row, &label, &section_fmt);
                write_table_header(worksheet, row, &["波长(nm)", "吸光度"], &header_fmt);
                row += 1;
                for i in 0..spec.values.len() {
                    let fmt = if i % 2 == 0 { &value_fmt } else { &alt_value_fmt };
                    let wl = spec.wavelengths.get(i).copied().unwrap_or(0.0);
                    let val = spec.values.get(i).copied().unwrap_or(0.0);
                    worksheet
                        .write_with_format(row, 0, wl as f64, fmt)
                        .map_err(xlsx_err)?;
                    worksheet
                        .write_with_format(row, 1, val as f64, fmt)
                        .map_err(xlsx_err)?;
                    row += 1;
                }
                row += 1;
            }
        }
    }

    workbook.save(path).map_err(xlsx_err)?;
    Ok(())
}

// === 辅助函数 ===

/// 生成工作表名（最长 31 字符）
fn sheet_name(report: &StunnerReport) -> String {
    let name = report.filename();
    let name = name.trim_end_matches(".bin");
    if name.len() > 31 {
        name[..31].to_string()
    } else {
        name.to_string()
    }
}

/// Option<String> 转显示文本
fn opt_str(opt: &Option<String>) -> String {
    opt.as_deref().unwrap_or("-").to_string()
}

/// XlsxError 转 AppError
fn xlsx_err(e: XlsxError) -> AppError {
    AppError::ExportError(e.to_string())
}

// === 格式定义 ===

fn title_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(14)
        .set_font_color("#1F4E79")
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
}

fn section_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(11)
        .set_font_color("#2E75B6")
}

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(12)
        .set_font_color("#FFFFFF")
        .set_background_color("#4472C4")
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
        .set_border_color("#D9D9D9")
}

fn key_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(10)
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
        .set_border_color("#D9D9D9")
}

fn value_format() -> Format {
    Format::new()
        .set_font_name("Consolas")
        .set_font_size(10)
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
        .set_border_color("#D9D9D9")
}

fn alt_key_format() -> Format {
    key_format().set_background_color("#F2F2F2")
}

fn alt_value_format() -> Format {
    value_format().set_background_color("#F2F2F2")
}

// === 写入辅助 ===

fn write_section_header(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    text: &str,
    fmt: &Format,
) -> u32 {
    let _ = worksheet.write_with_format(row, 0, text, fmt);
    row + 1
}

fn write_table_header(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    headers: &[&str],
    fmt: &Format,
) {
    for (i, h) in headers.iter().enumerate() {
        let _ = worksheet.write_with_format(row, i as u16, *h, fmt);
    }
}

fn write_kv_rows(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    start_row: u32,
    items: &[(&str, String)],
    key_fmt: &Format,
    value_fmt: &Format,
    alt_key_fmt: &Format,
    alt_value_fmt: &Format,
) -> u32 {
    let mut row = start_row;
    for (i, (key, val)) in items.iter().enumerate() {
        let kf = if i % 2 == 0 { key_fmt } else { alt_key_fmt };
        let vf = if i % 2 == 0 { value_fmt } else { alt_value_fmt };
        let _ = worksheet.write_with_format(row, 0, *key, kf);
        let _ = worksheet.write_with_format(row, 1, val.as_str(), vf);
        row += 1;
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{FileParser, StunnerParser};
    use std::path::Path;

    #[test]
    fn test_export_xlsx_from_real_bin() {
        let bin_path = Path::new(
            "../unchained-labs公司bin文件实验数据解析/\
             2026-06-16_160331_901294_ADMI_123/\
             2026-06-16_104823_901294_ADMI_123.bin",
        );
        if !bin_path.exists() {
            eprintln!("跳过测试：测试 bin 文件不存在");
            return;
        }

        let parser = StunnerParser::new();
        let report = parser.parse_file(bin_path).expect("解析失败");

        // 验证解析结果
        assert!(!report.filename().is_empty());
        assert!(
            !report.spectra.is_empty(),
            "应至少提取到一条光谱数据"
        );

        // 导出到临时 xlsx
        let out_path = std::env::temp_dir().join("stunner_test_export.xlsx");
        export_xlsx(&[report], &out_path).expect("导出失败");
        assert!(out_path.exists(), "xlsx 文件应已生成");

        // 清理
        let _ = std::fs::remove_file(&out_path);
    }
}
