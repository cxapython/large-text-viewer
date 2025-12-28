mod app;
mod i18n;
mod plugin;
mod plugins;
mod trace_plugin;
mod trace_analyzer;
mod trace_mode;
#[cfg(feature = "python-plugins")]
mod python_plugin;

use app::TextViewerApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // 初始化 Python 插件系统
    #[cfg(feature = "python-plugins")]
    {
        if let Err(e) = python_plugin::PythonPluginLoader::init_python() {
            eprintln!("警告: Python 插件系统初始化失败: {}", e);
        }
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("大文本查看器"),
        ..Default::default()
    };

    eframe::run_native(
        "大文本查看器",
        options,
        Box::new(|cc| {
            // 配置中文字体
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(TextViewerApp::default()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 尝试加载系统中文字体
    // macOS: PingFang SC, Heiti SC
    // Windows: Microsoft YaHei, SimHei
    // Linux: Noto Sans CJK SC, WenQuanYi Micro Hei

    let font_paths = if cfg!(target_os = "macos") {
        vec![
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
        ]
    } else {
        vec![
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        ]
    };

    let mut font_loaded = false;
    for font_path in font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                egui::FontData::from_owned(font_data).into(),
            );

            // 将中文字体添加到所有字体族的首位
            if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                proportional.insert(0, "chinese_font".to_owned());
            }
            if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                monospace.insert(0, "chinese_font".to_owned());
            }

            font_loaded = true;
            println!("已加载中文字体: {}", font_path);
            break;
        }
    }

    if !font_loaded {
        eprintln!("警告: 未能加载中文字体，中文可能显示为方块");
    }

    ctx.set_fonts(fonts);
}
