//! ARM64 Trace 分析插件
//!
//! 专业的 QBDI Trace 文件分析插件，提供：
//! - 语法高亮
//! - 指令类型过滤
//! - 寄存器追踪
//! - 统计信息
//! - 数据流分析

use eframe::egui;
use std::collections::HashMap;
use regex::Regex;

use crate::plugin::{Plugin, PluginInfo, PluginContext, PluginPanelTheme};
use crate::i18n::Language;

// ========== 常量定义 ==========

const PLUGIN_INFO: PluginInfo = PluginInfo {
    id: "arm64_trace",
    name: "ARM64 Trace Analyzer",
    name_cn: "ARM64 Trace 分析器",
    description: "Professional analyzer for QBDI trace files",
    description_cn: "专业的 QBDI Trace 文件分析工具",
    version: "1.0.0",
    author: "Large Text Viewer",
    icon: "🔬",
};

// ========== 数据类型 ==========

/// 指令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionType {
    Arithmetic,  // [A]
    Logic,       // [L]
    Memory,      // [M]
    Branch,      // [B]
    Call,        // [C]
    Return,      // [R]
    Unknown,
}

impl InstructionType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "[A]" => Self::Arithmetic,
            "[L]" => Self::Logic,
            "[M]" => Self::Memory,
            "[B]" => Self::Branch,
            "[C]" => Self::Call,
            "[R]" => Self::Return,
            _ => Self::Unknown,
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            Self::Arithmetic => "[A]",
            Self::Logic => "[L]",
            Self::Memory => "[M]",
            Self::Branch => "[B]",
            Self::Call => "[C]",
            Self::Return => "[R]",
            Self::Unknown => "[?]",
        }
    }

    pub fn name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Arithmetic, Language::Chinese) => "算术",
            (Self::Arithmetic, _) => "Arithmetic",
            (Self::Logic, Language::Chinese) => "逻辑",
            (Self::Logic, _) => "Logic",
            (Self::Memory, Language::Chinese) => "内存",
            (Self::Memory, _) => "Memory",
            (Self::Branch, Language::Chinese) => "分支",
            (Self::Branch, _) => "Branch",
            (Self::Call, Language::Chinese) => "调用",
            (Self::Call, _) => "Call",
            (Self::Return, Language::Chinese) => "返回",
            (Self::Return, _) => "Return",
            (Self::Unknown, Language::Chinese) => "未知",
            (Self::Unknown, _) => "Unknown",
        }
    }

    pub fn all() -> &'static [InstructionType] {
        &[
            Self::Arithmetic,
            Self::Logic,
            Self::Memory,
            Self::Branch,
            Self::Call,
            Self::Return,
        ]
    }
}

/// 解析后的指令
#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub seq_num: u64,
    pub depth: u8,
    pub inst_type: InstructionType,
    pub address: u64,
    pub offset: u64,
    pub disasm: String,
    pub reg_changes: Vec<RegisterChange>,
}

/// 寄存器变化
#[derive(Debug, Clone)]
pub struct RegisterChange {
    pub reg_name: String,
    pub old_value: u64,
    pub new_value: u64,
}

/// 统计信息
#[derive(Debug, Clone, Default)]
pub struct TraceStatistics {
    pub total_instructions: usize,
    pub instruction_counts: HashMap<InstructionType, usize>,
    pub max_depth: u8,
    pub memory_reads: usize,
    pub memory_writes: usize,
    pub function_entries: usize,
    pub function_exits: usize,
}

/// 过滤器配置
#[derive(Debug, Clone)]
pub struct TraceFilter {
    pub enabled_types: HashMap<InstructionType, bool>,
    pub depth_filter: Option<u8>,
    pub depth_filter_enabled: bool,
    pub register_filter: Option<String>,
    pub register_filter_enabled: bool,
    pub seq_range: Option<(u64, u64)>,
    pub seq_filter_enabled: bool,
    pub show_memory_only: bool,
    pub show_reg_changes_only: bool,
}

impl Default for TraceFilter {
    fn default() -> Self {
        let mut enabled_types = HashMap::new();
        for t in InstructionType::all() {
            enabled_types.insert(*t, true);
        }
        Self {
            enabled_types,
            depth_filter: None,
            depth_filter_enabled: false,
            register_filter: None,
            register_filter_enabled: false,
            seq_range: None,
            seq_filter_enabled: false,
            show_memory_only: false,
            show_reg_changes_only: false,
        }
    }
}

// ========== 颜色主题 ==========

pub struct TraceColorTheme {
    pub arithmetic: egui::Color32,
    pub logic: egui::Color32,
    pub memory: egui::Color32,
    pub branch: egui::Color32,
    pub call: egui::Color32,
    pub return_: egui::Color32,
    pub seq_number: egui::Color32,
    pub depth: egui::Color32,
    pub address: egui::Color32,
    pub offset: egui::Color32,
    pub register: egui::Color32,
    pub register_value: egui::Color32,
    pub memory_read: egui::Color32,
    pub memory_write: egui::Color32,
    pub function_marker: egui::Color32,
    pub comment: egui::Color32,
    pub hex_value: egui::Color32,
}

impl Default for TraceColorTheme {
    fn default() -> Self {
        Self {
            arithmetic: egui::Color32::from_rgb(152, 195, 121),
            logic: egui::Color32::from_rgb(97, 175, 239),
            memory: egui::Color32::from_rgb(229, 192, 123),
            branch: egui::Color32::from_rgb(198, 120, 221),
            call: egui::Color32::from_rgb(224, 108, 117),
            return_: egui::Color32::from_rgb(255, 121, 198),
            seq_number: egui::Color32::from_rgb(86, 182, 194),
            depth: egui::Color32::from_rgb(198, 120, 221),
            address: egui::Color32::from_rgb(150, 150, 150),
            offset: egui::Color32::from_rgb(120, 120, 120),
            register: egui::Color32::from_rgb(255, 184, 108),
            register_value: egui::Color32::from_rgb(209, 154, 102),
            memory_read: egui::Color32::from_rgb(97, 175, 239),
            memory_write: egui::Color32::from_rgb(224, 108, 117),
            function_marker: egui::Color32::from_rgb(255, 215, 0),
            comment: egui::Color32::from_rgb(92, 99, 112),
            hex_value: egui::Color32::from_rgb(152, 195, 121),
        }
    }
}

impl TraceColorTheme {
    pub fn get_instruction_color(&self, inst_type: InstructionType) -> egui::Color32 {
        match inst_type {
            InstructionType::Arithmetic => self.arithmetic,
            InstructionType::Logic => self.logic,
            InstructionType::Memory => self.memory,
            InstructionType::Branch => self.branch,
            InstructionType::Call => self.call,
            InstructionType::Return => self.return_,
            InstructionType::Unknown => egui::Color32::GRAY,
        }
    }
}

// ========== 插件主体 ==========

pub struct TraceAnalyzerPlugin {
    // 解析器
    instruction_regex: Option<Regex>,
    reg_change_regex: Option<Regex>,
    mem_op_regex: Option<Regex>,
    enter_regex: Option<Regex>,
    leave_regex: Option<Regex>,
    
    // 状态
    filter: TraceFilter,
    statistics: Option<TraceStatistics>,
    theme: TraceColorTheme,
    
    // UI 状态
    show_filter_section: bool,
    show_stats_section: bool,
    show_quick_nav: bool,
    
    // 输入框
    depth_input: String,
    register_input: String,
    seq_start_input: String,
    seq_end_input: String,
}

impl Default for TraceAnalyzerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceAnalyzerPlugin {
    pub fn new() -> Self {
        Self {
            instruction_regex: None,
            reg_change_regex: None,
            mem_op_regex: None,
            enter_regex: None,
            leave_regex: None,
            filter: TraceFilter::default(),
            statistics: None,
            theme: TraceColorTheme::default(),
            show_filter_section: true,
            show_stats_section: true,
            show_quick_nav: false,
            depth_input: String::new(),
            register_input: String::new(),
            seq_start_input: String::new(),
            seq_end_input: String::new(),
        }
    }
    
    fn init_regex(&mut self) {
        if self.instruction_regex.is_none() {
            self.instruction_regex = Regex::new(
                r"^#(\d+)\s+\[D(\d+)\]\s+\[([ALMBC R])\]\s+0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)\s+(.+?)(?:;(.*))?$"
            ).ok();
            
            self.reg_change_regex = Regex::new(
                r"([XWS][0-9]+|SP|LR|PC)=0x([0-9a-fA-F]+)->0x([0-9a-fA-F]+)"
            ).ok();
            
            self.mem_op_regex = Regex::new(
                r"MEM_(read|write)\s+@0x([0-9a-fA-F]+)\s+size=(\d+)\s+val=([0-9a-fA-F]+)"
            ).ok();
            
            self.enter_regex = Regex::new(r"======\s+ENTER\s+0x([0-9a-fA-F]+)").ok();
            self.leave_regex = Regex::new(r"======\s+LEAVE\s+0x([0-9a-fA-F]+)").ok();
        }
    }
    
    fn parse_instruction(&mut self, line: &str) -> Option<ParsedInstruction> {
        self.init_regex();
        
        let regex = self.instruction_regex.as_ref()?;
        let caps = regex.captures(line)?;

        let seq_num = caps.get(1)?.as_str().parse().ok()?;
        let depth = caps.get(2)?.as_str().parse().ok()?;
        let type_tag = format!("[{}]", caps.get(3)?.as_str());
        let inst_type = InstructionType::from_tag(&type_tag);
        let address = u64::from_str_radix(caps.get(4)?.as_str(), 16).ok()?;
        let offset = u64::from_str_radix(caps.get(5)?.as_str(), 16).ok()?;
        let disasm = caps.get(6)?.as_str().trim().to_string();

        let mut reg_changes = Vec::new();
        if let Some(reg_str) = caps.get(7) {
            if let Some(ref reg_regex) = self.reg_change_regex {
                for cap in reg_regex.captures_iter(reg_str.as_str()) {
                    if let (Some(name), Some(old), Some(new)) = (cap.get(1), cap.get(2), cap.get(3)) {
                        if let (Ok(old_val), Ok(new_val)) = (
                            u64::from_str_radix(old.as_str(), 16),
                            u64::from_str_radix(new.as_str(), 16)
                        ) {
                            reg_changes.push(RegisterChange {
                                reg_name: name.as_str().to_string(),
                                old_value: old_val,
                                new_value: new_val,
                            });
                        }
                    }
                }
            }
        }

        Some(ParsedInstruction {
            seq_num,
            depth,
            inst_type,
            address,
            offset,
            disasm,
            reg_changes,
        })
    }
    
    fn compute_statistics(&mut self, content: &str) {
        let mut stats = TraceStatistics::default();
        let lines: Vec<&str> = content.lines().collect();
        let sample_size = 1000.min(lines.len());
        let step = if lines.len() > sample_size { lines.len() / sample_size } else { 1 };
        
        for (i, line) in lines.iter().enumerate() {
            // 函数入口/出口
            if line.contains("ENTER") && line.contains("======") {
                stats.function_entries += 1;
            }
            if line.contains("LEAVE") && line.contains("======") {
                stats.function_exits += 1;
            }
            
            // 内存操作
            if line.contains("MEM_read") {
                stats.memory_reads += 1;
            }
            if line.contains("MEM_write") {
                stats.memory_writes += 1;
            }
            
            if i % step != 0 { continue; }
            
            if let Some(inst) = self.parse_instruction(line) {
                stats.total_instructions += 1;
                *stats.instruction_counts.entry(inst.inst_type).or_insert(0) += 1;
                stats.max_depth = stats.max_depth.max(inst.depth);
            }
        }
        
        // 估算
        if step > 1 {
            let scale = lines.len() as f64 / sample_size as f64;
            stats.total_instructions = (stats.total_instructions as f64 * scale) as usize;
            for count in stats.instruction_counts.values_mut() {
                *count = (*count as f64 * scale) as usize;
            }
        }
        
        self.statistics = Some(stats);
    }
    
    fn reset_filter(&mut self) {
        self.filter = TraceFilter::default();
        self.depth_input.clear();
        self.register_input.clear();
        self.seq_start_input.clear();
        self.seq_end_input.clear();
    }
    
    fn apply_filter_settings(&mut self) {
        if !self.depth_input.is_empty() {
            if let Ok(d) = self.depth_input.parse::<u8>() {
                self.filter.depth_filter = Some(d);
            }
        } else {
            self.filter.depth_filter = None;
        }

        if !self.seq_start_input.is_empty() || !self.seq_end_input.is_empty() {
            let start = self.seq_start_input.parse::<u64>().unwrap_or(0);
            let end = self.seq_end_input.parse::<u64>().unwrap_or(u64::MAX);
            self.filter.seq_range = Some((start, end));
        } else {
            self.filter.seq_range = None;
        }

        if !self.register_input.is_empty() {
            self.filter.register_filter = Some(self.register_input.clone());
        } else {
            self.filter.register_filter = None;
        }
    }
}

// ========== Plugin Trait 实现 ==========

impl Plugin for TraceAnalyzerPlugin {
    fn info(&self) -> &PluginInfo {
        &PLUGIN_INFO
    }
    
    fn can_handle(&self, content_sample: &str) -> bool {
        let first_lines: String = content_sample.lines().take(50).collect::<Vec<_>>().join("\n");
        first_lines.contains("QBDI Trace")
            || first_lines.contains("[hook] target=")
            || (first_lines.contains("[D") && first_lines.contains("] ["))
            || first_lines.contains("====== ENTER")
    }
    
    fn on_activate(&mut self, content_sample: &str) {
        self.init_regex();
        self.compute_statistics(content_sample);
        self.show_filter_section = true;
        self.show_stats_section = true;
    }
    
    fn on_deactivate(&mut self) {
        self.statistics = None;
        self.reset_filter();
    }
    
    fn render_side_panel(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        let theme = if ctx.dark_mode {
            PluginPanelTheme::dark()
        } else {
            PluginPanelTheme::light()
        };
        let lang = ctx.language;
        
        ui.vertical(|ui| {
            // 插件标题
            render_plugin_header(ui, &theme, lang);
            
            ui.add_space(12.0);
            
            // 统计信息区块
            if let Some(ref stats) = self.statistics {
                render_statistics_section(ui, stats, &self.theme, &theme, lang, &mut self.show_stats_section);
            }
            
            ui.add_space(8.0);
            
            // 过滤器区块
            let filter_changed = render_filter_section(
                ui, 
                &mut self.filter,
                &self.theme,
                &theme,
                lang,
                &mut self.show_filter_section,
                &mut self.depth_input,
                &mut self.register_input,
                &mut self.seq_start_input,
                &mut self.seq_end_input,
            );
            
            // 当过滤设置变化时应用设置
            if filter_changed {
                self.apply_filter_settings();
            }
            
            ui.add_space(12.0);
            
            // 重置按钮
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui.add(
                    egui::Button::new(egui::RichText::new(
                        if lang == Language::Chinese { "🔄 重置过滤器" } else { "🔄 Reset Filter" }
                    ).size(12.0))
                    .fill(theme.section_bg)
                    .rounding(6.0)
                ).clicked() {
                    self.reset_filter();
                }
            });
        });
    }
    
    fn render_line(&mut self, ui: &mut egui::Ui, line: &str, ctx: &PluginContext) {
        let mut job = egui::text::LayoutJob::default();
        let font_id = egui::FontId::monospace(ctx.font_size);
        let trimmed = line.trim_start();
        let default_color = egui::Color32::from_rgb(171, 178, 191);

        // 注释行
        if trimmed.starts_with("# ") || trimmed.starts_with("//") {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: self.theme.comment,
                ..Default::default()
            });
            ui.label(job);
            return;
        }

        // 函数入口/出口
        if trimmed.contains("======") {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: self.theme.function_marker,
                background: egui::Color32::from_rgba_unmultiplied(255, 215, 0, 20),
                ..Default::default()
            });
            ui.label(job);
            return;
        }

        // Hook 信息
        if trimmed.starts_with("[hook]") || trimmed.starts_with("[gqb]") {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: egui::Color32::from_rgb(86, 182, 194),
                ..Default::default()
            });
            ui.label(job);
            return;
        }

        // 内存操作行
        if trimmed.starts_with("MEM_") {
            highlight_memory_line(&mut job, line, &font_id, &self.theme);
            ui.label(job);
            return;
        }

        // SRC_REG 行
        if trimmed.starts_with("SRC_REG") {
            highlight_src_reg_line(&mut job, line, &font_id, &self.theme);
            ui.label(job);
            return;
        }

        // 指令行
        if let Some(inst) = self.parse_instruction(line) {
            highlight_instruction_line(&mut job, line, &inst, &font_id, &self.theme);
        } else {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: default_color,
                ..Default::default()
            });
        }

        ui.label(job);
    }
}

// ========== 渲染辅助函数 ==========

fn render_plugin_header(ui: &mut egui::Ui, theme: &PluginPanelTheme, lang: Language) {
    egui::Frame::none()
        .fill(theme.header_bg)
        .rounding(8.0)
        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔬").size(20.0));
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(
                        if lang == Language::Chinese { "ARM64 Trace 分析器" } else { "ARM64 Trace Analyzer" }
                    )
                    .size(14.0)
                    .strong()
                    .color(theme.text_color));
                    ui.label(egui::RichText::new("v1.0.0")
                        .size(10.0)
                        .color(theme.text_muted));
                });
            });
        });
}

fn render_statistics_section(
    ui: &mut egui::Ui,
    stats: &TraceStatistics,
    trace_theme: &TraceColorTheme,
    panel_theme: &PluginPanelTheme,
    lang: Language,
    expanded: &mut bool,
) {
    // 标题栏
    let header = ui.horizontal(|ui| {
        ui.add_space(4.0);
        let arrow = if *expanded { "▼" } else { "▶" };
        ui.label(egui::RichText::new(arrow).size(10.0).color(panel_theme.text_muted));
        ui.label(egui::RichText::new("📊").size(13.0));
        ui.label(egui::RichText::new(
            if lang == Language::Chinese { "统计信息" } else { "Statistics" }
        ).size(12.0).strong().color(panel_theme.accent_color));
    });
    
    if header.response.interact(egui::Sense::click()).clicked() {
        *expanded = !*expanded;
    }
    
    if !*expanded {
        return;
    }
    
    ui.add_space(4.0);
    
    egui::Frame::none()
        .fill(panel_theme.section_bg)
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            // 主要统计
            ui.horizontal_wrapped(|ui| {
                // 总指令
                stat_badge(ui, 
                    if lang == Language::Chinese { "指令" } else { "Instr" },
                    &format_number(stats.total_instructions),
                    panel_theme.accent_color,
                    panel_theme,
                );
                
                // 最大深度
                stat_badge(ui,
                    if lang == Language::Chinese { "深度" } else { "Depth" },
                    &format!("D{}", stats.max_depth),
                    trace_theme.depth,
                    panel_theme,
                );
                
                // 内存操作
                stat_badge(ui,
                    "R/W",
                    &format!("{}/{}", format_number(stats.memory_reads), format_number(stats.memory_writes)),
                    trace_theme.memory,
                    panel_theme,
                );
            });
            
            ui.add_space(6.0);
            
            // 指令类型分布
            ui.horizontal_wrapped(|ui| {
                for inst_type in InstructionType::all() {
                    let count = stats.instruction_counts.get(inst_type).copied().unwrap_or(0);
                    if count > 0 {
                        let pct = if stats.total_instructions > 0 {
                            count as f32 / stats.total_instructions as f32 * 100.0
                        } else {
                            0.0
                        };
                        let color = trace_theme.get_instruction_color(*inst_type);
                        ui.label(egui::RichText::new(format!(
                            "{}{:.0}%",
                            inst_type.tag(),
                            pct
                        )).size(10.0).monospace().color(color));
                    }
                }
            });
            
            // 函数统计
            if stats.function_entries > 0 || stats.function_exits > 0 {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!(
                    "{}: {} / {}",
                    if lang == Language::Chinese { "函数" } else { "Func" },
                    stats.function_entries,
                    stats.function_exits
                )).size(10.0).color(trace_theme.function_marker));
            }
        });
}

fn stat_badge(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    color: egui::Color32,
    theme: &PluginPanelTheme,
) {
    egui::Frame::none()
        .fill(theme.hover_bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(label).size(9.0).color(theme.text_muted));
                ui.label(egui::RichText::new(value).size(11.0).strong().color(color));
            });
        });
}

fn render_filter_section(
    ui: &mut egui::Ui,
    filter: &mut TraceFilter,
    trace_theme: &TraceColorTheme,
    panel_theme: &PluginPanelTheme,
    lang: Language,
    expanded: &mut bool,
    depth_input: &mut String,
    register_input: &mut String,
    seq_start_input: &mut String,
    seq_end_input: &mut String,
) -> bool {
    let mut settings_changed = false;
    
    // 标题栏
    let header = ui.horizontal(|ui| {
        ui.add_space(4.0);
        let arrow = if *expanded { "▼" } else { "▶" };
        ui.label(egui::RichText::new(arrow).size(10.0).color(panel_theme.text_muted));
        ui.label(egui::RichText::new("🔍").size(13.0));
        ui.label(egui::RichText::new(
            if lang == Language::Chinese { "过滤器" } else { "Filter" }
        ).size(12.0).strong().color(panel_theme.accent_color));
    });
    
    if header.response.interact(egui::Sense::click()).clicked() {
        *expanded = !*expanded;
    }
    
    if !*expanded {
        return false;
    }
    
    ui.add_space(4.0);
    
    // 指令类型过滤
    egui::Frame::none()
        .fill(panel_theme.section_bg)
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(
                if lang == Language::Chinese { "📋 指令类型" } else { "📋 Instruction Type" }
            ).size(11.0).color(panel_theme.text_muted));
            
            ui.add_space(4.0);
            
            let types = [
                (InstructionType::Arithmetic, trace_theme.arithmetic),
                (InstructionType::Logic, trace_theme.logic),
                (InstructionType::Memory, trace_theme.memory),
                (InstructionType::Branch, trace_theme.branch),
                (InstructionType::Call, trace_theme.call),
                (InstructionType::Return, trace_theme.return_),
            ];
            
            ui.horizontal_wrapped(|ui| {
                for (inst_type, color) in types.iter() {
                    let enabled = filter.enabled_types.get(inst_type).copied().unwrap_or(true);
                    let tag = inst_type.tag();
                    
                    let btn = egui::Button::new(
                        egui::RichText::new(tag)
                            .size(10.0)
                            .monospace()
                            .color(if enabled { *color } else { panel_theme.text_muted })
                    )
                    .fill(if enabled { panel_theme.active_bg } else { panel_theme.hover_bg })
                    .rounding(4.0);
                    
                    if ui.add(btn).clicked() {
                        filter.enabled_types.insert(*inst_type, !enabled);
                        settings_changed = true;
                    }
                }
            });
        });
    
    ui.add_space(4.0);
    
    // 深度过滤
    egui::Frame::none()
        .fill(panel_theme.section_bg)
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut filter.depth_filter_enabled, "").changed() {
                    settings_changed = true;
                }
                ui.label(egui::RichText::new(
                    if lang == Language::Chinese { "🏷️ 深度" } else { "🏷️ Depth" }
                ).size(11.0).color(panel_theme.text_muted));
            });
            
            if filter.depth_filter_enabled {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("[D").color(trace_theme.depth).monospace().size(11.0));
                    let response = ui.add(
                        egui::TextEdit::singleline(depth_input)
                            .desired_width(35.0)
                            .font(egui::FontId::monospace(11.0))
                    );
                    ui.label(egui::RichText::new("]").color(trace_theme.depth).monospace().size(11.0));
                    
                    // 当输入变化或按回车时应用设置
                    if response.changed() || response.lost_focus() {
                        settings_changed = true;
                    }
                });
            }
        });
    
    ui.add_space(4.0);
    
    // 寄存器追踪
    egui::Frame::none()
        .fill(panel_theme.section_bg)
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut filter.register_filter_enabled, "").changed() {
                    settings_changed = true;
                }
                ui.label(egui::RichText::new(
                    if lang == Language::Chinese { "📝 寄存器" } else { "📝 Register" }
                ).size(11.0).color(panel_theme.text_muted));
            });
            
            if filter.register_filter_enabled {
                // 快捷按钮
                ui.horizontal_wrapped(|ui| {
                    for reg in &["X0", "X1", "X8", "SP", "LR"] {
                        if ui.add(egui::Button::new(
                            egui::RichText::new(*reg).monospace().size(9.0)
                        ).fill(panel_theme.hover_bg).rounding(3.0)).clicked() {
                            *register_input = reg.to_string();
                            filter.register_filter = Some(reg.to_string());
                            settings_changed = true;
                        }
                    }
                });
                
                ui.add_space(2.0);
                let response = ui.add(
                    egui::TextEdit::singleline(register_input)
                        .hint_text("X0, X1...")
                        .desired_width(ui.available_width() - 4.0)
                        .font(egui::FontId::monospace(11.0))
                );
                
                if response.changed() || response.lost_focus() {
                    settings_changed = true;
                }
            }
        });
    
    ui.add_space(4.0);
    
    // 序号范围
    egui::Frame::none()
        .fill(panel_theme.section_bg)
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut filter.seq_filter_enabled, "").changed() {
                    settings_changed = true;
                }
                ui.label(egui::RichText::new(
                    if lang == Language::Chinese { "🔢 序号范围" } else { "🔢 Seq Range" }
                ).size(11.0).color(panel_theme.text_muted));
            });
            
            if filter.seq_filter_enabled {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("#").color(trace_theme.seq_number).monospace().size(11.0));
                    let r1 = ui.add(
                        egui::TextEdit::singleline(seq_start_input)
                            .hint_text("start")
                            .desired_width(50.0)
                            .font(egui::FontId::monospace(10.0))
                    );
                    ui.label(egui::RichText::new("-").size(11.0));
                    let r2 = ui.add(
                        egui::TextEdit::singleline(seq_end_input)
                            .hint_text("end")
                            .desired_width(50.0)
                            .font(egui::FontId::monospace(10.0))
                    );
                    
                    if r1.changed() || r1.lost_focus() || r2.changed() || r2.lost_focus() {
                        settings_changed = true;
                    }
                });
            }
        });
    
    ui.add_space(4.0);
    
    // 其他选项
    egui::Frame::none()
        .fill(panel_theme.section_bg)
        .rounding(6.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(
                if lang == Language::Chinese { "⚙️ 选项" } else { "⚙️ Options" }
            ).size(11.0).color(panel_theme.text_muted));
            
            ui.add_space(2.0);
            
            if ui.checkbox(
                &mut filter.show_memory_only,
                egui::RichText::new(
                    if lang == Language::Chinese { "仅内存操作" } else { "Memory only" }
                ).size(11.0)
            ).changed() {
                settings_changed = true;
            }
            
            if ui.checkbox(
                &mut filter.show_reg_changes_only,
                egui::RichText::new(
                    if lang == Language::Chinese { "仅寄存器变化" } else { "Reg changes only" }
                ).size(11.0)
            ).changed() {
                settings_changed = true;
            }
        });
    
    settings_changed
}

// ========== 高亮辅助函数 ==========

fn highlight_instruction_line(
    job: &mut egui::text::LayoutJob,
    line: &str,
    inst: &ParsedInstruction,
    font_id: &egui::FontId,
    theme: &TraceColorTheme,
) {
    let default_color = egui::Color32::from_rgb(171, 178, 191);
    let mut pos = 0;

    // 序号 #xxx
    if let Some(hash_pos) = line.find('#') {
        if hash_pos > 0 {
            job.append(&line[..hash_pos], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
        }
        
        let seq_end = line[hash_pos..].find(' ').map(|i| hash_pos + i).unwrap_or(line.len());
        job.append(&line[hash_pos..seq_end], 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: theme.seq_number,
            ..Default::default()
        });
        pos = seq_end;
    }

    // [Dx] 深度
    if let Some(d_start) = line[pos..].find("[D") {
        let d_start = pos + d_start;
        if d_start > pos {
            job.append(&line[pos..d_start], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
        }
        
        if let Some(d_end) = line[d_start..].find(']') {
            let d_end = d_start + d_end + 1;
            job.append(&line[d_start..d_end], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: theme.depth,
                ..Default::default()
            });
            pos = d_end;
        }
    }

    // [X] 指令类型
    let type_markers = ["[A]", "[L]", "[M]", "[B]", "[C]", "[R]"];
    for marker in type_markers {
        if let Some(t_start) = line[pos..].find(marker) {
            let t_start = pos + t_start;
            if t_start > pos {
                job.append(&line[pos..t_start], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: default_color,
                    ..Default::default()
                });
            }
            
            let t_end = t_start + 3;
            let color = theme.get_instruction_color(inst.inst_type);
            job.append(&line[t_start..t_end], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            });
            pos = t_end;
            break;
        }
    }

    // 地址和偏移
    let remaining = &line[pos..];
    if let Some(addr_match) = remaining.find("0x") {
        let addr_start = pos + addr_match;
        if addr_start > pos {
            job.append(&line[pos..addr_start], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
        }
        
        let addr_end = find_hex_end(&line[addr_start..]) + addr_start;
        job.append(&line[addr_start..addr_end], 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: theme.address,
            ..Default::default()
        });
        pos = addr_end;
        
        // 第二个地址（偏移）
        let remaining = &line[pos..];
        if let Some(off_match) = remaining.find("0x") {
            let off_start = pos + off_match;
            job.append(&line[pos..off_start], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
            
            let off_end = find_hex_end(&line[off_start..]) + off_start;
            job.append(&line[off_start..off_end], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: theme.offset,
                ..Default::default()
            });
            pos = off_end;
        }
    }

    // 指令和寄存器变化
    if let Some(semi_pos) = line[pos..].find(';') {
        let semi_pos = pos + semi_pos;
        // 指令部分
        highlight_instruction_text(job, &line[pos..semi_pos], font_id, inst.inst_type, theme);
        
        // 分号
        job.append(";", 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: egui::Color32::from_rgb(150, 150, 150),
            ..Default::default()
        });
        
        // 寄存器变化部分
        highlight_register_changes(job, &line[semi_pos + 1..], font_id, theme);
    } else {
        // 只有指令，没有寄存器变化
        highlight_instruction_text(job, &line[pos..], font_id, inst.inst_type, theme);
    }
}

fn highlight_instruction_text(
    job: &mut egui::text::LayoutJob,
    text: &str,
    font_id: &egui::FontId,
    inst_type: InstructionType,
    theme: &TraceColorTheme,
) {
    let inst_color = theme.get_instruction_color(inst_type);
    let default_color = egui::Color32::from_rgb(171, 178, 191);
    
    let trimmed = text.trim_start();
    let leading_space = text.len() - text.trim_start().len();
    
    if leading_space > 0 {
        job.append(&text[..leading_space], 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: default_color,
            ..Default::default()
        });
    }
    
    // 分割指令助记符和操作数
    if let Some(space_pos) = trimmed.find(|c: char| c.is_whitespace()) {
        let mnemonic = &trimmed[..space_pos];
        let operands = &trimmed[space_pos..];
        
        // 指令助记符
        job.append(mnemonic, 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: inst_color,
            ..Default::default()
        });
        
        // 操作数
        highlight_operands(job, operands, font_id, theme);
    } else {
        job.append(trimmed, 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: inst_color,
            ..Default::default()
        });
    }
}

fn highlight_operands(
    job: &mut egui::text::LayoutJob, 
    text: &str, 
    font_id: &egui::FontId,
    theme: &TraceColorTheme,
) {
    let default_color = egui::Color32::from_rgb(171, 178, 191);
    let mut chars = text.char_indices().peekable();
    let mut last_end = 0;

    while let Some((i, c)) = chars.next() {
        // 检查寄存器
        if c == 'x' || c == 'X' || c == 'w' || c == 'W' || c == 's' || c == 'S' {
            if let Some(&(_, next_c)) = chars.peek() {
                if next_c.is_ascii_digit() {
                    if i > last_end {
                        job.append(&text[last_end..i], 0.0, egui::TextFormat {
                            font_id: font_id.clone(),
                            color: default_color,
                            ..Default::default()
                        });
                    }
                    
                    let mut reg_end = i + 1;
                    while let Some(&(j, c)) = chars.peek() {
                        if c.is_ascii_digit() {
                            reg_end = j + 1;
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    
                    job.append(&text[i..reg_end], 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: theme.register,
                        ..Default::default()
                    });
                    last_end = reg_end;
                    continue;
                }
            }
        }
        
        // 检查 SP, LR, PC
        let remaining = &text[i..];
        for special in &["sp", "SP", "lr", "LR", "pc", "PC"] {
            if remaining.starts_with(special) {
                let after = remaining.get(special.len()..special.len() + 1).unwrap_or("");
                if after.is_empty() || !after.chars().next().unwrap().is_alphanumeric() {
                    if i > last_end {
                        job.append(&text[last_end..i], 0.0, egui::TextFormat {
                            font_id: font_id.clone(),
                            color: default_color,
                            ..Default::default()
                        });
                    }
                    
                    job.append(*special, 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: theme.register,
                        ..Default::default()
                    });
                    
                    for _ in 0..special.len() - 1 {
                        chars.next();
                    }
                    last_end = i + special.len();
                    break;
                }
            }
        }
        
        // 检查立即数 #0x...
        if c == '#' {
            if i > last_end {
                job.append(&text[last_end..i], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: default_color,
                    ..Default::default()
                });
            }
            
            let mut imm_end = i + 1;
            while let Some(&(j, c)) = chars.peek() {
                if c.is_alphanumeric() || c == 'x' || c == 'X' {
                    imm_end = j + 1;
                    chars.next();
                } else {
                    break;
                }
            }
            
            job.append(&text[i..imm_end], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: theme.hex_value,
                ..Default::default()
            });
            last_end = imm_end;
        }
    }
    
    if last_end < text.len() {
        job.append(&text[last_end..], 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: default_color,
            ..Default::default()
        });
    }
}

fn highlight_register_changes(
    job: &mut egui::text::LayoutJob, 
    text: &str, 
    font_id: &egui::FontId,
    theme: &TraceColorTheme,
) {
    let mut pos = 0;
    
    while pos < text.len() {
        if let Some(eq_pos) = text[pos..].find('=') {
            let eq_pos = pos + eq_pos;
            let reg_name = &text[pos..eq_pos];
            
            job.append(reg_name, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: theme.register,
                ..Default::default()
            });
            
            job.append("=", 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(150, 150, 150),
                ..Default::default()
            });
            
            pos = eq_pos + 1;
            
            if let Some(arrow_pos) = text[pos..].find("->") {
                let arrow_pos = pos + arrow_pos;
                let old_val = &text[pos..arrow_pos];
                
                job.append(old_val, 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: theme.register_value,
                    ..Default::default()
                });
                
                job.append("->", 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(255, 100, 100),
                    ..Default::default()
                });
                
                pos = arrow_pos + 2;
                
                let new_end = text[pos..].find(char::is_whitespace).map(|i| pos + i).unwrap_or(text.len());
                let new_val = &text[pos..new_end];
                
                job.append(new_val, 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(100, 255, 100),
                    ..Default::default()
                });
                
                pos = new_end;
            } else {
                break;
            }
        } else {
            job.append(&text[pos..], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(171, 178, 191),
                ..Default::default()
            });
            break;
        }
    }
}

fn highlight_memory_line(
    job: &mut egui::text::LayoutJob, 
    line: &str, 
    font_id: &egui::FontId,
    theme: &TraceColorTheme,
) {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let default_color = egui::Color32::from_rgb(171, 178, 191);
    
    if indent > 0 {
        job.append(&line[..indent], 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: default_color,
            ..Default::default()
        });
    }
    
    let (op_color, op_end) = if trimmed.starts_with("MEM_read") {
        (theme.memory_read, indent + 8)
    } else if trimmed.starts_with("MEM_write") {
        (theme.memory_write, indent + 9)
    } else {
        (egui::Color32::GRAY, indent)
    };
    
    if op_end > indent {
        job.append(&line[indent..op_end], 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: op_color,
            ..Default::default()
        });
    }
    
    let remaining = &line[op_end..];
    let mut pos = 0;
    
    while pos < remaining.len() {
        if let Some(at_pos) = remaining[pos..].find('@') {
            job.append(&remaining[pos..pos + at_pos], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
            
            job.append("@", 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(150, 150, 150),
                ..Default::default()
            });
            
            pos = pos + at_pos + 1;
            
            if remaining[pos..].starts_with("0x") {
                let addr_end = find_hex_end(&remaining[pos..]) + pos;
                job.append(&remaining[pos..addr_end], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: theme.address,
                    ..Default::default()
                });
                pos = addr_end;
            }
        } else if let Some(val_pos) = remaining[pos..].find("val=") {
            job.append(&remaining[pos..pos + val_pos], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
            
            job.append("val=", 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(150, 150, 150),
                ..Default::default()
            });
            
            pos = pos + val_pos + 4;
            
            let val_end = remaining[pos..].find(char::is_whitespace).map(|i| pos + i).unwrap_or(remaining.len());
            job.append(&remaining[pos..val_end], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: theme.hex_value,
                ..Default::default()
            });
            pos = val_end;
        } else {
            job.append(&remaining[pos..], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
            break;
        }
    }
}

fn highlight_src_reg_line(
    job: &mut egui::text::LayoutJob, 
    line: &str, 
    font_id: &egui::FontId,
    theme: &TraceColorTheme,
) {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let default_color = egui::Color32::from_rgb(171, 178, 191);
    
    if indent > 0 {
        job.append(&line[..indent], 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: default_color,
            ..Default::default()
        });
    }
    
    if let Some(eq_pos) = trimmed.find('=') {
        job.append("SRC_REG", 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: egui::Color32::from_rgb(198, 120, 221),
            ..Default::default()
        });
        
        job.append("=", 0.0, egui::TextFormat {
            font_id: font_id.clone(),
            color: egui::Color32::from_rgb(150, 150, 150),
            ..Default::default()
        });
        
        let after_eq = &trimmed[eq_pos + 1..];
        if let Some(space_pos) = after_eq.find(' ') {
            let reg_name = &after_eq[..space_pos];
            job.append(reg_name, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: theme.register,
                ..Default::default()
            });
            
            let rest = &after_eq[space_pos..];
            if let Some(val_pos) = rest.find("val=") {
                job.append(&rest[..val_pos], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: default_color,
                    ..Default::default()
                });
                
                job.append("val=", 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(150, 150, 150),
                    ..Default::default()
                });
                
                let val = &rest[val_pos + 4..];
                job.append(val, 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: theme.hex_value,
                    ..Default::default()
                });
            } else {
                job.append(rest, 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: default_color,
                    ..Default::default()
                });
            }
        }
    }
}

// ========== 辅助函数 ==========

fn find_hex_end(s: &str) -> usize {
    let mut end = 0;
    let chars: Vec<char> = s.chars().collect();
    
    if s.starts_with("0x") || s.starts_with("0X") {
        end = 2;
    }
    
    while end < chars.len() && chars[end].is_ascii_hexdigit() {
        end += 1;
    }
    
    end
}

fn format_number(n: usize) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{:.1}K", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}