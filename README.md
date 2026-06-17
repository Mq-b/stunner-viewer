# Stunner Viewer

Unchained Labs Stunner `.bin` 文件解析器与查看器。

## 功能

- **打开 bin 文件** — 解析 TLV 格式元数据与光谱数据
- **光谱渲染** — 190-790nm 波长范围的 UV-Vis 吸收光谱折线图
- **信息展示** — 实验信息、仪器信息、测量位置列表
- **导出 XLSX** — 格式化 Excel 文件（蓝色表头、交替行、分区布局）
- **Fluent 主题** — 自动跟随系统 dark/light 主题

## 运行

```bash
cargo run
```

## 测试

```bash
cargo test
```

## 技术栈

- **Slint** — 声明式 UI（Fluent 主题）
- **byteorder** — 二进制解析
- **rust_xlsxwriter** — XLSX 导出
- **rfd** — 原生文件对话框
- **chrono** — FILETIME 时间转换

## 项目结构

```
src/
├── main.rs          # 应用入口（15 行）
├── app.rs           # 回调绑定与数据转换
├── exporter.rs      # XLSX 导出
├── error.rs         # 统一错误类型
├── model/           # 数据模型（StunnerReport 等）
└── parser/          # bin 文件解析（TLV 编解码 + 光谱提取）
ui/
├── main.slint       # 主窗口布局
└── spectrum-chart.slint  # 光谱折线图组件
assets/
├── icon.svg         # 矢量图标
├── icon.ico         # Windows 图标
└── app.rc           # Windows 资源文件
```
