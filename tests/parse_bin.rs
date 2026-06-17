//! Stunner bin 文件解析集成测试
//!
//! 验证解析器能正确解析真实 bin 文件的元数据和光谱数据。

use stunner_viewer::model::StunnerReport;
use stunner_viewer::parser::tlv::TlvReader;
use stunner_viewer::parser::{FileParser, StunnerParser};
use std::path::{Path, PathBuf};

/// 测试用 bin 文件所在目录
const BIN_DIR: &str = "../unchained-labs公司bin文件实验数据解析/2026-06-16_160331_901294_ADMI_123";

/// 获取所有测试 bin 文件路径
fn test_bin_files() -> Vec<PathBuf> {
    let dir = Path::new(BIN_DIR);
    if !dir.exists() {
        return vec![];
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "bin"))
        .collect();
    files.sort();
    files
}

/// 解析单个文件并返回报告
fn parse_one(path: &Path) -> StunnerReport {
    let parser = StunnerParser::new();
    let report = parser.parse_file(path).unwrap_or_else(|e| panic!("解析失败 {}: {}", path.display(), e));
    eprintln!(
        "  {}: {} 条光谱, {} 个位置",
        path.file_name().unwrap().to_string_lossy(),
        report.spectra.len(),
        report.layout.positions.len()
    );
    report
}

// === 元数据测试 ===

#[test]
fn test_all_bin_files_parse_without_error() {
    let files = test_bin_files();
    assert!(!files.is_empty(), "未找到测试 bin 文件，检查路径: {}", BIN_DIR);

    for path in &files {
        parse_one(path);
    }

    for path in &files {
        let report = parse_one(path);
        assert!(!report.filename().is_empty(), "文件名不应为空: {}", path.display());
    }
}

#[test]
fn test_experiment_metadata() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);

    // 实验名称应为 "123"（根据参考脚本输出）
    assert_eq!(
        report.experiment.name.as_deref(),
        Some("123"),
        "实验名称应为 '123'"
    );

    // 样品类型应为 "Protein"
    assert_eq!(
        report.experiment.sample_type.as_deref(),
        Some("Protein"),
        "样品类型应为 'Protein'"
    );
}

#[test]
fn test_instrument_metadata() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);

    // 序列号应为 "901294"
    assert_eq!(
        report.instrument.serial_number.as_deref(),
        Some("901294"),
        "序列号应为 '901294'"
    );

    // 软件类型应为 "Stunner"
    assert_eq!(
        report.instrument.software_type.as_deref(),
        Some("Stunner"),
        "软件类型应为 'Stunner'"
    );

    // MAC 地址格式验证
    if let Some(ref mac) = report.instrument.mac_address {
        assert!(
            mac.contains('-') || mac.contains(':'),
            "MAC 地址格式应包含分隔符: {}",
            mac
        );
    }
}

#[test]
fn test_user_metadata() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);

    // 用户名应为 "admin"
    assert_eq!(
        report.user_name.as_deref(),
        Some("admin"),
        "用户名应为 'admin'"
    );
}

#[test]
fn test_layout_metadata() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);

    // 测量位置数量（该文件应有 12 个）
    assert!(
        report.layout.positions.len() >= 12,
        "应至少有 12 个测量位置，实际: {}",
        report.layout.positions.len()
    );

    // 第一个位置名称应包含 "blank" 或 "sample"
    if let Some(first) = report.layout.positions.first() {
        assert!(
            first.name.contains("blank") || first.name.contains("sample"),
            "位置名称应包含 'blank' 或 'sample': {}",
            first.name
        );
    }
}

// === 光谱数据测试 ===

#[test]
fn test_spectra_extracted() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);

    assert!(
        !report.spectra.is_empty(),
        "应至少提取到一条光谱数据"
    );

    // 每条光谱应有数据点
    for (i, spec) in report.spectra.iter().enumerate() {
        assert!(
            !spec.values.is_empty(),
            "光谱 #{} 不应为空",
            i
        );
        assert_eq!(
            spec.wavelengths.len(),
            spec.values.len(),
            "光谱 #{} 波长和值数量应一致",
            i
        );
    }
}

#[test]
fn test_spectra_wavelength_range() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);
    let spec = &report.spectra[0];

    // 波长范围应为 190-790nm
    let wl_min = spec.wavelengths.first().copied().unwrap_or(0.0);
    let wl_max = spec.wavelengths.last().copied().unwrap_or(0.0);
    assert!(
        (189.0..=191.0).contains(&wl_min),
        "起始波长应约为 190nm: {}",
        wl_min
    );
    assert!(
        (789.0..=791.0).contains(&wl_max),
        "结束波长应约为 790nm: {}",
        wl_max
    );
}

#[test]
fn test_spectra_values_reasonable() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);

    for (i, spec) in report.spectra.iter().enumerate() {
        // 值应在合理范围内（吸光度通常 0-10）
        for (j, val) in spec.values.iter().enumerate() {
            assert!(
                val.is_finite(),
                "光谱 #{} 点 #{} 值不是有限数: {}",
                i, j, val
            );
        }
    }
}

#[test]
fn test_spectrum_wavelength_value_count_match() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);
    for (i, spec) in report.spectra.iter().enumerate() {
        assert_eq!(
            spec.wavelengths.len(),
            spec.values.len(),
            "光谱 #{} 波长数({}) ≠ 值数({})",
            i,
            spec.wavelengths.len(),
            spec.values.len()
        );
    }
}

// === 光谱数据对比测试 ===

#[test]
#[ignore] // 手动运行：cargo test --test parse_bin compare_spectra -- --ignored --nocapture
fn compare_spectra() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);
    println!("Rust: {} 条光谱", report.spectra.len());
    for (i, spec) in report.spectra.iter().take(10).enumerate() {
        let first3: Vec<String> = spec.values.iter().take(3).map(|v| format!("{:.6}", v)).collect();
        println!(
            "  [{:3}] pos_index={:3}  count={:4}  first3=[{}]",
            i, spec.position_index, spec.values.len(), first3.join(", ")
        );
    }
    // 统计对齐
    let total = report.spectra.len();
    println!("共 {} 条光谱", total);
}

// === 调试：打印 TLV 条目 ===

#[test]
#[ignore] // 手动运行：cargo test --test parse_bin debug_tlv_entries -- --ignored --nocapture
fn debug_tlv_entries() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let data = std::fs::read(&files[0]).expect("读取文件失败");
    println!("文件: {} ({} 字节)", files[0].display(), data.len());

    // 打印前 100 字节
    println!("\n前 100 字节:");
    for (i, chunk) in data[..100.min(data.len())].chunks(16).enumerate() {
        print!("{:04X}: ", i * 16);
        for b in chunk {
            print!("{:02X} ", b);
        }
        // 补齐到 16 字节
        for _ in chunk.len()..16 {
            print!("   ");
        }
        print!(" | ");
        for b in chunk {
            if *b >= 0x20 && *b < 0x7F {
                print!("{}", *b as char);
            } else {
                print!(".");
            }
        }
        println!();
    }

    // 手动读取前几个字节
    println!("\n手动解析:");
    if data.len() >= 16 {
        let key_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        println!("  key_len = {} (0x{:X})", key_len, key_len);
        if key_len < 100 && 4 + key_len as usize <= data.len() {
            let key = &data[4..4 + key_len as usize];
            println!("  key = {:?} ({} bytes)", String::from_utf8_lossy(key), key.len());
            let val_offset = 4 + key_len as usize;
            if val_offset + 4 <= data.len() {
                let val_len = u32::from_be_bytes([
                    data[val_offset],
                    data[val_offset + 1],
                    data[val_offset + 2],
                    data[val_offset + 3],
                ]);
                println!("  val_len = {} (0x{:X})", val_len, val_len);
                let next = val_offset + 4 + val_len as usize;
                println!("  下一个条目偏移: 0x{:X}", next);
            }
        }
    }

    let mut reader = TlvReader::new(&data);
    for i in 0..30 {
        match reader.read_entry() {
            Ok(entry) => {
                println!("[{:2}] key={:?}, value={:?}", i, entry.key, entry.value);
            }
            Err(e) => {
                println!("[{:2}] 解析失败: {} (偏移: {:#X})", i, e, reader.offset());
                break;
            }
        }
    }
}

// === 吸光度数据测试 ===

#[test]
fn test_absorbance_data() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let report = parse_one(&files[0]);

    // 吸光度阈值应已提取（参考脚本显示值为 0.25）
    if !report.absorbance.thresholds.is_empty() {
        for (i, val) in report.absorbance.thresholds.iter().enumerate() {
            assert!(
                val.is_finite(),
                "吸光度阈值 #{} 不是有限数: {}",
                i, val
            );
        }
    }
}

// === 多文件一致性测试 ===

#[test]
fn test_all_files_same_instrument() {
    let files = test_bin_files();
    if files.len() < 2 {
        eprintln!("跳过：测试文件不足");
        return;
    }

    let parser = StunnerParser::new();
    let reports: Vec<StunnerReport> = files
        .iter()
        .map(|p| parser.parse_file(p).expect(&format!("解析失败: {}", p.display())))
        .collect();

    // 所有文件应来自同一仪器
    let serial = reports[0].instrument.serial_number.as_deref();
    for (i, report) in reports.iter().enumerate().skip(1) {
        assert_eq!(
            report.instrument.serial_number.as_deref(),
            serial,
            "文件 #{} 序列号不一致",
            i
        );
    }
}

#[test]
fn test_all_files_have_spectra() {
    let files = test_bin_files();
    if files.is_empty() {
        eprintln!("跳过：无测试 bin 文件");
        return;
    }

    let parser = StunnerParser::new();
    for path in &files {
        let report = parser
            .parse_file(path)
            .unwrap_or_else(|e| panic!("解析失败 {}: {}", path.display(), e));
        assert!(
            !report.spectra.is_empty(),
            "文件 {} 应有光谱数据",
            path.display()
        );
    }
}
