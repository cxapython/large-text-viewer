//! Python 插件支持模块
//!
//! 提供 Python 3 插件加载和执行能力，允许用户使用 Python 编写插件扩展功能。
//!
//! ## 用法
//! 
//! 将 Python 插件放在 `~/.large-text-viewer/plugins/` 目录下，应用启动时会自动加载。
//!
//! ## Python 插件 API
//!
//! Python 插件需要实现 `LTVPlugin` 类，包含以下方法：
//! - `info()` - 返回插件信息字典
//! - `can_handle(content_sample: str) -> bool` - 检测是否可以处理此文件
//! - `on_activate(content_sample: str)` - 插件激活时调用
//! - `on_deactivate()` - 插件停用时调用
//! - `get_panel_data() -> dict` - 返回侧边面板数据
//! - `process_line(line: str) -> list` - 处理单行，返回高亮片段

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use eframe::egui;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::{Deserialize, Serialize};

use crate::plugin::{Plugin, PluginInfo, PluginContext, PluginPanelTheme};
use crate::i18n::Language;

// ========== Python 插件数据类型 ==========

/// Python 插件信息（运行时动态获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyPluginInfo {
    pub id: String,
    pub name: String,
    pub name_cn: String,
    pub description: String,
    pub description_cn: String,
    pub version: String,
    pub author: String,
    pub icon: String,
}

impl Default for PyPluginInfo {
    fn default() -> Self {
        Self {
            id: "unknown".to_string(),
            name: "Unknown Plugin".to_string(),
            name_cn: "未知插件".to_string(),
            description: "No description".to_string(),
            description_cn: "无描述".to_string(),
            version: "0.0.0".to_string(),
            author: "Unknown".to_string(),
            icon: "🔌".to_string(),
        }
    }
}

/// 高亮文本片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSegment {
    pub text: String,
    pub color: Option<[u8; 3]>,  // RGB
    pub bold: bool,
    pub italic: bool,
}

impl Default for HighlightSegment {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: None,
            bold: false,
            italic: false,
        }
    }
}

/// 面板数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelItem {
    pub item_type: String,  // "label", "stat", "button", "separator", "section"
    pub label: String,
    pub value: Option<String>,
    pub color: Option<[u8; 3]>,
    pub action: Option<String>,  // 按钮动作标识
}

/// 面板数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PanelData {
    pub items: Vec<PanelItem>,
}

// ========== Python 插件包装器 ==========

/// Python 插件包装器
/// 
/// 将 Python 插件适配到 Rust 的 Plugin trait
pub struct PythonPlugin {
    /// 插件文件路径
    path: PathBuf,
    /// Python 插件实例（惰性加载）
    instance: Arc<Mutex<Option<Py<PyAny>>>>,
    /// 缓存的插件信息
    cached_info: PyPluginInfo,
    /// 缓存的面板数据
    cached_panel_data: Arc<Mutex<PanelData>>,
    /// 静态插件信息（用于 trait 返回引用）
    static_info: PluginInfo,
    /// 是否已激活
    is_active: bool,
}

// ========== 静态字符串存储 ==========
// 用于将动态字符串转换为 'static 引用（通过泄漏内存）

fn leak_string(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

impl PythonPlugin {
    /// 从 Python 文件创建插件
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let path = path.to_path_buf();
        
        // 尝试加载并获取插件信息
        let info = Python::with_gil(|py| {
            Self::load_plugin_info(py, &path)
        }).map_err(|e| format!("加载 Python 插件失败: {}", e))?;
        
        // 创建静态信息
        let static_info = PluginInfo {
            id: leak_string(&info.id),
            name: leak_string(&info.name),
            name_cn: leak_string(&info.name_cn),
            description: leak_string(&info.description),
            description_cn: leak_string(&info.description_cn),
            version: leak_string(&info.version),
            author: leak_string(&info.author),
            icon: leak_string(&info.icon),
        };
        
        Ok(Self {
            path,
            instance: Arc::new(Mutex::new(None)),
            cached_info: info,
            cached_panel_data: Arc::new(Mutex::new(PanelData::default())),
            static_info,
            is_active: false,
        })
    }
    
    /// 从 Python 文件加载插件信息
    fn load_plugin_info(py: Python<'_>, path: &Path) -> PyResult<PyPluginInfo> {
        // 读取 Python 文件内容
        let code = std::fs::read_to_string(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        
        // 获取文件所在目录
        let parent_dir = path.parent().unwrap_or(Path::new("."));
        
        // 添加插件目录到 Python 路径
        let sys = py.import_bound("sys")?;
        let sys_path = sys.getattr("path")?;
        let sys_path = sys_path.downcast::<PyList>()?;
        sys_path.insert(0, parent_dir.to_string_lossy().to_string())?;
        
        // 执行 Python 代码
        let module = PyModule::from_code_bound(
            py,
            &code,
            path.file_name().unwrap().to_str().unwrap(),
            path.file_stem().unwrap().to_str().unwrap(),
        )?;
        
        // 获取 LTVPlugin 类
        let plugin_class = module.getattr("LTVPlugin")?;
        
        // 创建实例并获取信息
        let instance = plugin_class.call0()?;
        let info_dict = instance.call_method0("info")?;
        
        // 解析信息字典
        let info = Self::parse_plugin_info(&info_dict)?;
        
        Ok(info)
    }
    
    /// 解析 Python 字典为 PyPluginInfo
    fn parse_plugin_info(dict: &Bound<'_, PyAny>) -> PyResult<PyPluginInfo> {
        let dict = dict.downcast::<PyDict>()?;
        
        let get_str = |key: &str, default: &str| -> String {
            dict.get_item(key)
                .ok()
                .flatten()
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_else(|| default.to_string())
        };
        
        Ok(PyPluginInfo {
            id: get_str("id", "unknown"),
            name: get_str("name", "Unknown Plugin"),
            name_cn: get_str("name_cn", "未知插件"),
            description: get_str("description", "No description"),
            description_cn: get_str("description_cn", "无描述"),
            version: get_str("version", "0.0.0"),
            author: get_str("author", "Unknown"),
            icon: get_str("icon", "🔌"),
        })
    }
    
    /// 确保 Python 实例已加载
    fn ensure_instance(&self) -> Result<(), String> {
        let mut instance_guard = self.instance.lock().map_err(|e| e.to_string())?;
        
        if instance_guard.is_none() {
            Python::with_gil(|py| {
                let code = std::fs::read_to_string(&self.path)
                    .map_err(|e| e.to_string())?;
                
                let parent_dir = self.path.parent().unwrap_or(Path::new("."));
                let sys = py.import_bound("sys").map_err(|e: PyErr| e.to_string())?;
                let sys_path = sys.getattr("path")
                    .map_err(|e: PyErr| e.to_string())?;
                let sys_path = sys_path.downcast::<PyList>()
                    .map_err(|e| e.to_string())?;
                sys_path.insert(0, parent_dir.to_string_lossy().to_string())
                    .map_err(|e: PyErr| e.to_string())?;
                
                let module = PyModule::from_code_bound(
                    py,
                    &code,
                    self.path.file_name().unwrap().to_str().unwrap(),
                    self.path.file_stem().unwrap().to_str().unwrap(),
                ).map_err(|e: PyErr| e.to_string())?;
                
                let plugin_class = module.getattr("LTVPlugin").map_err(|e: PyErr| e.to_string())?;
                let inst = plugin_class.call0().map_err(|e: PyErr| e.to_string())?;
                
                *instance_guard = Some(inst.into_py(py));
                Ok::<_, String>(())
            })?;
        }
        
        Ok(())
    }
    
    /// 调用无返回值的方法
    fn call_method_void(&self, method: &str) -> Result<(), String> {
        self.ensure_instance()?;
        
        let instance_guard = self.instance.lock().map_err(|e| e.to_string())?;
        let instance = instance_guard.as_ref().ok_or("插件实例未初始化")?;
        
        Python::with_gil(|py| {
            let inst = instance.bind(py);
            inst.call_method0(method)
                .map_err(|e| format!("调用 {} 失败: {}", method, e))?;
            Ok(())
        })
    }
    
    /// 调用带参数无返回值的方法
    fn call_method1_void(&self, method: &str, arg: &str) -> Result<(), String> {
        self.ensure_instance()?;
        
        let instance_guard = self.instance.lock().map_err(|e| e.to_string())?;
        let instance = instance_guard.as_ref().ok_or("插件实例未初始化")?;
        
        Python::with_gil(|py| {
            let inst = instance.bind(py);
            inst.call_method1(method, (arg,))
                .map_err(|e| format!("调用 {} 失败: {}", method, e))?;
            Ok(())
        })
    }
    
    /// 获取面板数据
    fn fetch_panel_data(&self) -> PanelData {
        let result = Python::with_gil(|py| -> Result<PanelData, String> {
            self.ensure_instance()?;
            
            let instance_guard = self.instance.lock().map_err(|e| e.to_string())?;
            let instance = instance_guard.as_ref().ok_or("插件实例未初始化")?;
            let inst = instance.bind(py);
            
            let result = inst.call_method0("get_panel_data")
                .map_err(|e| format!("get_panel_data 失败: {}", e))?;
            
            Self::parse_panel_data(py, &result)
        });
        
        result.unwrap_or_default()
    }
    
    /// 解析面板数据
    fn parse_panel_data(_py: Python<'_>, data: &Bound<'_, PyAny>) -> Result<PanelData, String> {
        let dict = data.downcast::<PyDict>().map_err(|e| e.to_string())?;
        let mut panel_data = PanelData::default();
        
        if let Some(items) = dict.get_item("items").ok().flatten() {
            if let Ok(items_list) = items.downcast::<PyList>() {
                for item in items_list.iter() {
                    if let Ok(item_dict) = item.downcast::<PyDict>() {
                        let panel_item = PanelItem {
                            item_type: item_dict.get_item("type").ok().flatten()
                                .and_then(|v| v.extract::<String>().ok())
                                .unwrap_or_else(|| "label".to_string()),
                            label: item_dict.get_item("label").ok().flatten()
                                .and_then(|v| v.extract::<String>().ok())
                                .unwrap_or_default(),
                            value: item_dict.get_item("value").ok().flatten()
                                .and_then(|v| v.extract::<String>().ok()),
                            color: item_dict.get_item("color").ok().flatten()
                                .and_then(|v| v.extract::<[u8; 3]>().ok()),
                            action: item_dict.get_item("action").ok().flatten()
                                .and_then(|v| v.extract::<String>().ok()),
                        };
                        panel_data.items.push(panel_item);
                    }
                }
            }
        }
        
        Ok(panel_data)
    }
    
    /// 处理行高亮
    fn process_line_highlight(&self, line: &str) -> Vec<HighlightSegment> {
        let result = Python::with_gil(|py| -> Result<Vec<HighlightSegment>, String> {
            self.ensure_instance()?;
            
            let instance_guard = self.instance.lock().map_err(|e| e.to_string())?;
            let instance = instance_guard.as_ref().ok_or("插件实例未初始化")?;
            let inst = instance.bind(py);
            
            let result = inst.call_method1("process_line", (line,))
                .map_err(|e| format!("process_line 失败: {}", e))?;
            
            Self::parse_highlight_segments(py, &result)
        });
        
        result.unwrap_or_else(|_| vec![HighlightSegment {
            text: line.to_string(),
            color: None,
            bold: false,
            italic: false,
        }])
    }
    
    /// 解析高亮片段
    fn parse_highlight_segments(_py: Python<'_>, data: &Bound<'_, PyAny>) -> Result<Vec<HighlightSegment>, String> {
        let list = data.downcast::<PyList>().map_err(|e| e.to_string())?;
        let mut segments = Vec::new();
        
        for item in list.iter() {
            if let Ok(item_dict) = item.downcast::<PyDict>() {
                let segment = HighlightSegment {
                    text: item_dict.get_item("text").ok().flatten()
                        .and_then(|v| v.extract::<String>().ok())
                        .unwrap_or_default(),
                    color: item_dict.get_item("color").ok().flatten()
                        .and_then(|v| v.extract::<[u8; 3]>().ok()),
                    bold: item_dict.get_item("bold").ok().flatten()
                        .and_then(|v| v.extract::<bool>().ok())
                        .unwrap_or(false),
                    italic: item_dict.get_item("italic").ok().flatten()
                        .and_then(|v| v.extract::<bool>().ok())
                        .unwrap_or(false),
                };
                segments.push(segment);
            }
        }
        
        Ok(segments)
    }
}

// ========== Plugin Trait 实现 ==========

impl Plugin for PythonPlugin {
    fn info(&self) -> &PluginInfo {
        &self.static_info
    }
    
    fn can_handle(&self, content_sample: &str) -> bool {
        Python::with_gil(|py| -> bool {
            if let Err(_) = self.ensure_instance() {
                return false;
            }
            
            let instance_guard = match self.instance.lock() {
                Ok(g) => g,
                Err(_) => return false,
            };
            
            let instance = match instance_guard.as_ref() {
                Some(i) => i,
                None => return false,
            };
            
            let inst = instance.bind(py);
            inst.call_method1("can_handle", (content_sample,))
                .ok()
                .and_then(|r| r.extract::<bool>().ok())
                .unwrap_or(false)
        })
    }
    
    fn on_activate(&mut self, content_sample: &str) {
        self.is_active = true;
        let _ = self.call_method1_void("on_activate", content_sample);
        
        // 更新缓存的面板数据
        let panel_data = self.fetch_panel_data();
        if let Ok(mut guard) = self.cached_panel_data.lock() {
            *guard = panel_data;
        }
    }
    
    fn on_deactivate(&mut self) {
        self.is_active = false;
        let _ = self.call_method_void("on_deactivate");
    }
    
    fn render_side_panel(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        let theme = if ctx.dark_mode {
            PluginPanelTheme::dark()
        } else {
            PluginPanelTheme::light()
        };
        
        // 获取最新的面板数据
        let panel_data = self.fetch_panel_data();
        
        ui.vertical(|ui| {
            // 插件标题
            egui::Frame::none()
                .fill(theme.header_bg)
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&self.cached_info.icon).size(20.0));
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            let name = if ctx.language == Language::Chinese {
                                &self.cached_info.name_cn
                            } else {
                                &self.cached_info.name
                            };
                            ui.label(egui::RichText::new(name)
                                .size(14.0)
                                .strong()
                                .color(theme.text_color));
                            ui.label(egui::RichText::new(format!("v{}", self.cached_info.version))
                                .size(10.0)
                                .color(theme.text_muted));
                        });
                    });
                });
            
            ui.add_space(12.0);
            
            // 渲染面板项
            for item in &panel_data.items {
                match item.item_type.as_str() {
                    "separator" => {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                    }
                    "section" => {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&item.label)
                            .size(12.0)
                            .strong()
                            .color(theme.accent_color));
                        ui.add_space(4.0);
                    }
                    "stat" => {
                        egui::Frame::none()
                            .fill(theme.section_bg)
                            .rounding(4.0)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&item.label)
                                        .size(10.0)
                                        .color(theme.text_muted));
                                    if let Some(value) = &item.value {
                                        let color = item.color
                                            .map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]))
                                            .unwrap_or(theme.accent_color);
                                        ui.label(egui::RichText::new(value)
                                            .size(12.0)
                                            .strong()
                                            .color(color));
                                    }
                                });
                            });
                    }
                    "button" => {
                        if ui.add(egui::Button::new(egui::RichText::new(&item.label).size(12.0))
                            .fill(theme.section_bg)
                            .rounding(6.0)
                        ).clicked() {
                            if let Some(action) = &item.action {
                                let _ = self.call_method1_void("on_action", action);
                            }
                        }
                    }
                    _ => {
                        // label 或其他
                        let color = item.color
                            .map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]))
                            .unwrap_or(theme.text_color);
                        ui.label(egui::RichText::new(&item.label)
                            .size(11.0)
                            .color(color));
                    }
                }
            }
        });
    }
    
    fn render_line(&mut self, ui: &mut egui::Ui, line: &str, ctx: &PluginContext) {
        let segments = self.process_line_highlight(line);
        let font_id = egui::FontId::monospace(ctx.font_size);
        
        let mut job = egui::text::LayoutJob::default();
        
        for segment in segments {
            let color = segment.color
                .map(|c| egui::Color32::from_rgb(c[0], c[1], c[2]))
                .unwrap_or(egui::Color32::from_rgb(171, 178, 191));
            
            let mut format = egui::TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            };
            
            // 注意：egui 的 TextFormat 不直接支持 bold/italic
            // 如果需要，可以使用不同的字体
            
            job.append(&segment.text, 0.0, format);
        }
        
        ui.label(job);
    }
}

// ========== 插件加载器 ==========

/// Python 插件加载器
pub struct PythonPluginLoader;

impl PythonPluginLoader {
    /// 获取默认插件目录
    pub fn default_plugin_dir() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".large-text-viewer").join("plugins")
    }
    
    /// 从目录加载所有 Python 插件
    pub fn load_from_directory(dir: &Path) -> Vec<Box<dyn Plugin>> {
        let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
        
        if !dir.exists() {
            // 创建插件目录
            if let Err(e) = std::fs::create_dir_all(dir) {
                eprintln!("无法创建插件目录 {:?}: {}", dir, e);
                return plugins;
            }
        }
        
        // 遍历目录查找 .py 文件
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("无法读取插件目录 {:?}: {}", dir, e);
                return plugins;
            }
        };
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            // 只加载 .py 文件
            if path.extension().and_then(|s| s.to_str()) != Some("py") {
                continue;
            }
            
            // 跳过以 _ 开头的文件（如 __init__.py）
            if path.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with('_'))
                .unwrap_or(false)
            {
                continue;
            }
            
            match PythonPlugin::from_file(&path) {
                Ok(plugin) => {
                    println!("已加载 Python 插件: {} ({})", 
                        plugin.cached_info.name, 
                        path.display());
                    plugins.push(Box::new(plugin));
                }
                Err(e) => {
                    eprintln!("加载 Python 插件 {:?} 失败: {}", path, e);
                }
            }
        }
        
        plugins
    }
    
    /// 初始化 Python 环境
    pub fn init_python() -> Result<(), String> {
        Python::with_gil(|py| {
            // 确保 Python 已初始化
            let version = py.version();
            println!("Python 版本: {}", version);
            Ok(())
        })
    }
}

// 需要添加 dirs 依赖，或者使用其他方式获取 home 目录
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("USERPROFILE").ok().map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
    }
}

// ========== 测试模块 ==========

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_plugin_dir() {
        let dir = PythonPluginLoader::default_plugin_dir();
        assert!(dir.to_string_lossy().contains(".large-text-viewer"));
    }
}

