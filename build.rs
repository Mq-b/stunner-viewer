fn main() {
    // 编译 Slint UI 文件
    slint_build::compile("ui/main.slint").unwrap();

    // 编译 Windows 资源文件（图标等）
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }
}
