//! 插件系统核心模块
//!
//! 提供可扩展的插件架构，支持：
//! - 自定义文件类型处理
//! - 语法高亮
//! - 侧边面板
//! - 上下文菜单

use eframe::egui;
use std::collections::HashMap;
use crate::i18n::Language;

/// 插件操作结果
#[derive(Debug, Clone)]
pub enum PluginAction {
    /// 跳转到指定行
    GotoLine(usize),
    /// 复制文本到剪贴板
    CopyText(String),
    /// 显示消息
    ShowMessage(String),
    /// 无操作
    None,
}

/// 插件元信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub name_cn: &'static str,
    pub description: &'static str,
    pub description_cn: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub icon: &'static str,
}

/// 插件特征 - 所有插件必须实现
pub trait Plugin: Send {
    /// 获取插件信息
    fn info(&self) -> &PluginInfo;
    
    /// 检测是否可以处理此文件内容
    fn can_handle(&self, content_sample: &str) -> bool;
    
    /// 插件激活时调用
    fn on_activate(&mut self, content_sample: &str);
    
    /// 插件停用时调用
    fn on_deactivate(&mut self);
    
    /// 判断是否需要渲染侧边面板
    fn has_side_panel(&self) -> bool { true }
    
    /// 渲染侧边面板
    fn render_side_panel(&mut self, ui: &mut egui::Ui, ctx: &PluginContext);
    
    /// 渲染单行内容（带高亮）
    fn render_line(&mut self, ui: &mut egui::Ui, line: &str, ctx: &PluginContext);
    
    /// 渲染状态栏附加信息（可选）
    fn render_status_extra(&self, _ui: &mut egui::Ui, _ctx: &PluginContext) {}
    
    /// 渲染上下文菜单项（可选）
    fn context_menu_items(&mut self, _ui: &mut egui::Ui, _line: &str, _ctx: &PluginContext) -> Option<PluginAction> {
        None
    }
    
    /// 处理快捷键（可选）
    fn handle_shortcut(&mut self, _ctx: &egui::Context, _plugin_ctx: &PluginContext) -> Option<PluginAction> {
        None
    }
    
    /// 获取自定义设置（可选）
    fn settings(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    
    /// 应用设置（可选）
    fn apply_settings(&mut self, _settings: HashMap<String, String>) {}
}

/// 插件上下文 - 传递给插件的环境信息
#[derive(Clone)]
pub struct PluginContext {
    pub language: Language,
    pub font_size: f32,
    pub dark_mode: bool,
    pub panel_width: f32,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self {
            language: Language::English,
            font_size: 14.0,
            dark_mode: true,
            panel_width: 280.0,
        }
    }
}

/// 插件管理器
pub struct PluginManager {
    /// 已注册的插件
    plugins: Vec<Box<dyn Plugin>>,
    /// 当前激活的插件索引
    active_plugin_index: Option<usize>,
    /// 是否显示插件面板
    pub show_plugin_panel: bool,
    /// 插件面板宽度
    pub panel_width: f32,
    /// 插件上下文
    pub context: PluginContext,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            active_plugin_index: None,
            show_plugin_panel: true,
            panel_width: 300.0,
            context: PluginContext::default(),
        }
    }
    
    /// 注册插件
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    /// 获取插件数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
    
    /// 获取插件信息（按索引）
    pub fn get_plugin_info(&self, index: usize) -> Option<&PluginInfo> {
        self.plugins.get(index).map(|p| p.info())
    }
    
    /// 获取当前激活的插件索引
    pub fn active_index(&self) -> Option<usize> {
        self.active_plugin_index
    }
    
    /// 获取当前激活的插件信息
    pub fn active_plugin_info(&self) -> Option<&PluginInfo> {
        self.active_plugin_index.map(|i| self.plugins[i].info())
    }
    
    /// 检测并激活适合的插件
    pub fn detect_and_activate(&mut self, content_sample: &str) {
        // 停用当前插件
        if let Some(idx) = self.active_plugin_index {
            self.plugins[idx].on_deactivate();
        }
        
        // 查找可以处理此内容的插件
        for (i, plugin) in self.plugins.iter_mut().enumerate() {
            if plugin.can_handle(content_sample) {
                plugin.on_activate(content_sample);
                self.active_plugin_index = Some(i);
                self.show_plugin_panel = true;
                return;
            }
        }
        
        self.active_plugin_index = None;
    }
    
    /// 手动激活指定插件
    pub fn activate(&mut self, index: usize, content_sample: &str) {
        if index >= self.plugins.len() {
            return;
        }
        
        // 停用当前插件
        if let Some(idx) = self.active_plugin_index {
            self.plugins[idx].on_deactivate();
        }
        
        self.plugins[index].on_activate(content_sample);
        self.active_plugin_index = Some(index);
        self.show_plugin_panel = true;
    }
    
    /// 停用当前插件
    pub fn deactivate(&mut self) {
        if let Some(idx) = self.active_plugin_index {
            self.plugins[idx].on_deactivate();
            self.active_plugin_index = None;
        }
    }
    
    /// 是否有激活的插件
    pub fn has_active_plugin(&self) -> bool {
        self.active_plugin_index.is_some()
    }
    
    /// 渲染侧边面板
    pub fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(idx) = self.active_plugin_index {
            let ctx = self.context.clone();
            self.plugins[idx].render_side_panel(ui, &ctx);
        }
    }
    
    /// 渲染行内容
    pub fn render_line(&mut self, ui: &mut egui::Ui, line: &str) {
        if let Some(idx) = self.active_plugin_index {
            let ctx = self.context.clone();
            self.plugins[idx].render_line(ui, line, &ctx);
        }
    }
    
    /// 更新上下文
    pub fn update_context(&mut self, language: Language, font_size: f32, dark_mode: bool) {
        self.context.language = language;
        self.context.font_size = font_size;
        self.context.dark_mode = dark_mode;
        self.context.panel_width = self.panel_width;
    }
}

/// 插件面板颜色主题
pub struct PluginPanelTheme {
    pub header_bg: egui::Color32,
    pub section_bg: egui::Color32,
    pub accent_color: egui::Color32,
    pub text_color: egui::Color32,
    pub text_muted: egui::Color32,
    pub border_color: egui::Color32,
    pub hover_bg: egui::Color32,
    pub active_bg: egui::Color32,
}

impl Default for PluginPanelTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl PluginPanelTheme {
    pub fn dark() -> Self {
        Self {
            header_bg: egui::Color32::from_rgb(25, 27, 32),
            section_bg: egui::Color32::from_rgb(35, 38, 45),
            accent_color: egui::Color32::from_rgb(99, 179, 237),
            text_color: egui::Color32::from_rgb(230, 230, 235),
            text_muted: egui::Color32::from_rgb(140, 145, 155),
            border_color: egui::Color32::from_rgb(55, 58, 65),
            hover_bg: egui::Color32::from_rgb(45, 48, 55),
            active_bg: egui::Color32::from_rgb(50, 55, 65),
        }
    }
    
    pub fn light() -> Self {
        Self {
            header_bg: egui::Color32::from_rgb(245, 245, 248),
            section_bg: egui::Color32::from_rgb(255, 255, 255),
            accent_color: egui::Color32::from_rgb(59, 130, 246),
            text_color: egui::Color32::from_rgb(30, 30, 35),
            text_muted: egui::Color32::from_rgb(100, 100, 110),
            border_color: egui::Color32::from_rgb(220, 220, 225),
            hover_bg: egui::Color32::from_rgb(240, 240, 245),
            active_bg: egui::Color32::from_rgb(230, 235, 245),
        }
    }
}

/// 插件选择器返回的操作
pub enum PluginSelectorAction {
    None,
    Deactivate,
    Activate(usize),
    TogglePanel,
}

/// 渲染插件选择器（用于工具栏）- 返回需要执行的操作
pub fn render_plugin_selector(
    ui: &mut egui::Ui,
    manager: &PluginManager,
    language: Language,
) -> PluginSelectorAction {
    let theme = if ui.visuals().dark_mode {
        PluginPanelTheme::dark()
    } else {
        PluginPanelTheme::light()
    };
    
    let plugin_count = manager.plugin_count();
    let active_idx = manager.active_index();
    
    if plugin_count == 0 {
        return PluginSelectorAction::None;
    }
    
    // 收集插件信息（避免借用冲突）
    let plugin_infos: Vec<_> = (0..plugin_count)
        .filter_map(|i| manager.get_plugin_info(i).map(|info| (i, info.icon, info.name, info.name_cn)))
        .collect();
    
    let mut action = PluginSelectorAction::None;
    
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("🔌").size(14.0));
        
        let current_name = if let Some(idx) = active_idx {
            if let Some((_, _, name, name_cn)) = plugin_infos.iter().find(|(i, _, _, _)| *i == idx) {
                if language == Language::Chinese { *name_cn } else { *name }
            } else {
                if language == Language::Chinese { "选择插件..." } else { "Select Plugin..." }
            }
        } else {
            if language == Language::Chinese { "选择插件..." } else { "Select Plugin..." }
        };
        
        egui::ComboBox::from_id_salt("plugin_selector")
            .selected_text(egui::RichText::new(current_name).size(12.0))
            .width(150.0)
            .show_ui(ui, |ui| {
                // 无插件选项
                let is_none = active_idx.is_none();
                if ui.selectable_label(is_none, egui::RichText::new(
                    if language == Language::Chinese { "⭘ 无" } else { "⭘ None" }
                ).size(12.0)).clicked() {
                    action = PluginSelectorAction::Deactivate;
                }
                
                ui.separator();
                
                // 插件列表
                for (i, icon, name, name_cn) in &plugin_infos {
                    let is_active = active_idx == Some(*i);
                    let display_name = if language == Language::Chinese { *name_cn } else { *name };
                    let label = format!("{} {}", icon, display_name);
                    
                    if ui.selectable_label(is_active, egui::RichText::new(&label).size(12.0)).clicked() {
                        action = PluginSelectorAction::Activate(*i);
                    }
                }
            });
        
        // 显示/隐藏面板按钮
        if active_idx.is_some() {
            let panel_icon = if manager.show_plugin_panel { "◀" } else { "▶" };
            if ui.add(egui::Button::new(egui::RichText::new(panel_icon).size(12.0))
                .fill(theme.section_bg)
                .rounding(4.0)
            ).on_hover_text(if language == Language::Chinese { 
                if manager.show_plugin_panel { "隐藏面板" } else { "显示面板" }
            } else {
                if manager.show_plugin_panel { "Hide Panel" } else { "Show Panel" }
            }).clicked() {
                action = PluginSelectorAction::TogglePanel;
            }
        }
    });
    
    action
}

/// 渲染美观的分割线
pub fn render_section_divider(ui: &mut egui::Ui, theme: &PluginPanelTheme) {
    ui.add_space(8.0);
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    painter.line_segment(
        [
            egui::pos2(rect.left() + 10.0, rect.top()),
            egui::pos2(rect.right() - 10.0, rect.top()),
        ],
        egui::Stroke::new(1.0, theme.border_color),
    );
    ui.add_space(8.0);
}

/// 渲染统计卡片
pub fn render_stat_card(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    color: egui::Color32,
    theme: &PluginPanelTheme,
) {
    egui::Frame::none()
        .fill(theme.section_bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(label)
                    .size(10.0)
                    .color(theme.text_muted));
                ui.label(egui::RichText::new(value)
                    .size(14.0)
                    .strong()
                    .color(color));
            });
        });
}
