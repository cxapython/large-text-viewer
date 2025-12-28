//! Arm64Trace 模式 UI 组件
//!
//! 提供专业的 trace 文件查看界面：
//! - 语法高亮
//! - 过滤器面板
//! - 统计信息面板
//! - 数据流追踪

use eframe::egui;
use std::collections::HashMap;

use crate::trace_analyzer::{
    InstructionType, TraceAnalyzer, TraceFilter, TraceStatistics,
};
use crate::i18n::Language;

/// 颜色主题 - 专业的逆向工程风格
pub struct TraceColorTheme {
    // 指令类型颜色
    pub arithmetic: egui::Color32,  // [A]
    pub logic: egui::Color32,       // [L]
    pub memory: egui::Color32,      // [M]
    pub branch: egui::Color32,      // [B]
    pub call: egui::Color32,        // [C]
    pub return_: egui::Color32,     // [R]
    
    // 其他元素颜色
    pub seq_number: egui::Color32,     // 序号 #1234
    pub depth: egui::Color32,          // 深度 [D1]
    pub address: egui::Color32,        // 地址 0x...
    pub offset: egui::Color32,         // 偏移
    pub register: egui::Color32,       // 寄存器名
    pub register_value: egui::Color32, // 寄存器值
    pub memory_read: egui::Color32,    // 内存读
    pub memory_write: egui::Color32,   // 内存写
    pub function_marker: egui::Color32, // ENTER/LEAVE
    pub comment: egui::Color32,        // 注释
    pub hex_value: egui::Color32,      // 十六进制值
    
    // 背景色
    pub current_line_bg: egui::Color32,
    pub hover_line_bg: egui::Color32,
    pub memory_line_bg: egui::Color32,
}

impl Default for TraceColorTheme {
    fn default() -> Self {
        Self {
            // 指令类型 - 使用鲜明的颜色区分
            arithmetic: egui::Color32::from_rgb(152, 195, 121),  // 绿色
            logic: egui::Color32::from_rgb(97, 175, 239),        // 蓝色
            memory: egui::Color32::from_rgb(229, 192, 123),      // 金色/橙色
            branch: egui::Color32::from_rgb(198, 120, 221),      // 紫色
            call: egui::Color32::from_rgb(224, 108, 117),        // 红色
            return_: egui::Color32::from_rgb(255, 121, 198),     // 粉色
            
            // 其他元素
            seq_number: egui::Color32::from_rgb(86, 182, 194),   // 青色
            depth: egui::Color32::from_rgb(198, 120, 221),       // 紫色
            address: egui::Color32::from_rgb(150, 150, 150),     // 灰色
            offset: egui::Color32::from_rgb(120, 120, 120),      // 深灰
            register: egui::Color32::from_rgb(255, 184, 108),    // 橙色
            register_value: egui::Color32::from_rgb(209, 154, 102), // 浅橙
            memory_read: egui::Color32::from_rgb(97, 175, 239),  // 蓝色
            memory_write: egui::Color32::from_rgb(224, 108, 117), // 红色
            function_marker: egui::Color32::from_rgb(255, 215, 0), // 金色
            comment: egui::Color32::from_rgb(92, 99, 112),       // 灰色
            hex_value: egui::Color32::from_rgb(152, 195, 121),   // 绿色
            
            // 背景色
            current_line_bg: egui::Color32::from_rgb(45, 50, 60),
            hover_line_bg: egui::Color32::from_rgb(40, 44, 52),
            memory_line_bg: egui::Color32::from_rgb(35, 40, 48),
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

/// Trace 模式状态
pub struct TraceModeState {
    pub enabled: bool,
    pub filter: TraceFilter,
    pub statistics: Option<TraceStatistics>,
    pub theme: TraceColorTheme,
    pub analyzer: TraceAnalyzer,
    
    // UI 状态
    pub show_filter_panel: bool,
    pub show_stats_panel: bool,
    pub filter_panel_width: f32,
    pub stats_panel_collapsed: bool,
    
    // 过滤器输入
    pub depth_input: String,
    pub register_input: String,
    pub address_start_input: String,
    pub address_end_input: String,
    pub seq_start_input: String,
    pub seq_end_input: String,
    
    // 跳转输入
    pub goto_seq_input: String,
    pub goto_address_input: String,
    
    // 缓存的过滤状态
    cached_visible_lines: HashMap<usize, bool>,
    cache_valid: bool,
}

impl Default for TraceModeState {
    fn default() -> Self {
        Self {
            enabled: false,
            filter: TraceFilter::default(),
            statistics: None,
            theme: TraceColorTheme::default(),
            analyzer: TraceAnalyzer::new(),
            show_filter_panel: true,
            show_stats_panel: true,
            filter_panel_width: 280.0,
            stats_panel_collapsed: false,
            depth_input: String::new(),
            register_input: String::new(),
            address_start_input: String::new(),
            address_end_input: String::new(),
            seq_start_input: String::new(),
            seq_end_input: String::new(),
            goto_seq_input: String::new(),
            goto_address_input: String::new(),
            cached_visible_lines: HashMap::new(),
            cache_valid: false,
        }
    }
}

impl TraceModeState {
    /// 重置过滤器
    pub fn reset_filter(&mut self) {
        self.filter = TraceFilter::default();
        self.depth_input.clear();
        self.register_input.clear();
        self.address_start_input.clear();
        self.address_end_input.clear();
        self.seq_start_input.clear();
        self.seq_end_input.clear();
        self.invalidate_cache();
    }

    /// 使缓存失效
    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
        self.cached_visible_lines.clear();
    }

    /// 应用过滤器设置
    pub fn apply_filter_settings(&mut self) {
        // 解析深度
        if !self.depth_input.is_empty() {
            if let Ok(d) = self.depth_input.parse::<u8>() {
                self.filter.depth_filter = Some(d);
            }
        } else {
            self.filter.depth_filter = None;
        }

        // 解析序号范围
        if !self.seq_start_input.is_empty() || !self.seq_end_input.is_empty() {
            let start = self.seq_start_input.parse::<u64>().unwrap_or(0);
            let end = self.seq_end_input.parse::<u64>().unwrap_or(u64::MAX);
            self.filter.seq_range = Some((start, end));
        } else {
            self.filter.seq_range = None;
        }

        // 解析地址范围
        if !self.address_start_input.is_empty() || !self.address_end_input.is_empty() {
            let start = parse_hex_address(&self.address_start_input).unwrap_or(0);
            let end = parse_hex_address(&self.address_end_input).unwrap_or(u64::MAX);
            self.filter.address_range = Some((start, end));
        } else {
            self.filter.address_range = None;
        }

        // 寄存器过滤
        if !self.register_input.is_empty() {
            self.filter.register_filter = Some(self.register_input.clone());
        } else {
            self.filter.register_filter = None;
        }

        self.invalidate_cache();
    }

    /// 渲染带语法高亮的行
    pub fn render_highlighted_line(&mut self, ui: &mut egui::Ui, line: &str, font_size: f32) {
        let mut job = egui::text::LayoutJob::default();
        let font_id = egui::FontId::monospace(font_size);
        let trimmed = line.trim_start();

        // 注释行
        if trimmed.starts_with("# ") || trimmed.starts_with("//") {
            job.append(
                line,
                0.0,
                egui::TextFormat {
                    font_id,
                    color: self.theme.comment,
                    ..Default::default()
                },
            );
            ui.label(job);
            return;
        }

        // 函数入口/出口标记
        if trimmed.contains("======") {
            job.append(
                line,
                0.0,
                egui::TextFormat {
                    font_id,
                    color: self.theme.function_marker,
                    background: egui::Color32::from_rgba_unmultiplied(255, 215, 0, 20),
                    ..Default::default()
                },
            );
            ui.label(job);
            return;
        }

        // hook 信息
        if trimmed.starts_with("[hook]") || trimmed.starts_with("[gqb]") {
            job.append(
                line,
                0.0,
                egui::TextFormat {
                    font_id,
                    color: egui::Color32::from_rgb(86, 182, 194),
                    ..Default::default()
                },
            );
            ui.label(job);
            return;
        }

        // 内存操作行
        if trimmed.starts_with("MEM_") {
            self.highlight_memory_line(&mut job, line, &font_id);
            ui.label(job);
            return;
        }

        // SRC_REG 行
        if trimmed.starts_with("SRC_REG") {
            self.highlight_src_reg_line(&mut job, line, &font_id);
            ui.label(job);
            return;
        }

        // 指令行
        if let Some(inst) = self.analyzer.parse_instruction(line) {
            self.highlight_instruction_line(&mut job, line, &inst, &font_id);
        } else {
            // 普通行
            job.append(
                line,
                0.0,
                egui::TextFormat {
                    font_id,
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                },
            );
        }

        ui.label(job);
    }

    fn highlight_instruction_line(
        &self,
        job: &mut egui::text::LayoutJob,
        line: &str,
        inst: &crate::trace_analyzer::ParsedInstruction,
        font_id: &egui::FontId,
    ) {
        // 查找各个部分的位置
        let mut pos = 0;
        let _ = pos; // 将来用于复杂高亮

        // 序号 #xxx
        if let Some(hash_pos) = line.find('#') {
            // 序号前的空白
            if hash_pos > 0 {
                job.append(&line[..hash_pos], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
            }
            
            // 找序号结束位置
            let seq_end = line[hash_pos..].find(' ').map(|i| hash_pos + i).unwrap_or(line.len());
            job.append(&line[hash_pos..seq_end], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: self.theme.seq_number,
                ..Default::default()
            });
            pos = seq_end;
        }

        // [Dx] 深度
        if let Some(d_start) = line[pos..].find("[D") {
            let d_start = pos + d_start;
            // 中间的空白
            if d_start > pos {
                job.append(&line[pos..d_start], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
            }
            
            if let Some(d_end) = line[d_start..].find(']') {
                let d_end = d_start + d_end + 1;
                job.append(&line[d_start..d_end], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: self.theme.depth,
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
                // 中间的空白
                if t_start > pos {
                    job.append(&line[pos..t_start], 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: egui::Color32::from_rgb(171, 178, 191),
                        ..Default::default()
                    });
                }
                
                let t_end = t_start + 3;
                let color = self.theme.get_instruction_color(inst.inst_type);
                job.append(&line[t_start..t_end], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color,
                    ..Default::default()
                });
                pos = t_end;
                break;
            }
        }

        // 地址和偏移 0x...
        let remaining = &line[pos..];
        if let Some(addr_match) = remaining.find("0x") {
            let addr_start = pos + addr_match;
            // 空白
            if addr_start > pos {
                job.append(&line[pos..addr_start], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
            }
            
            // 第一个地址
            let addr_end = find_hex_end(&line[addr_start..]) + addr_start;
            job.append(&line[addr_start..addr_end], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: self.theme.address,
                ..Default::default()
            });
            pos = addr_end;
            
            // 第二个地址（偏移）
            let remaining = &line[pos..];
            if let Some(off_match) = remaining.find("0x") {
                let off_start = pos + off_match;
                // 分隔符
                job.append(&line[pos..off_start], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
                
                let off_end = find_hex_end(&line[off_start..]) + off_start;
                job.append(&line[off_start..off_end], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: self.theme.offset,
                    ..Default::default()
                });
                pos = off_end;
            }
        }

        // 指令和寄存器变化
        if let Some(semi_pos) = line[pos..].find(';') {
            let semi_pos = pos + semi_pos;
            // 指令部分
            let inst_part = &line[pos..semi_pos];
            self.highlight_instruction_text(job, inst_part, font_id, inst.inst_type);
            
            // 寄存器变化部分
            job.append(";", 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(150, 150, 150),
                ..Default::default()
            });
            
            let reg_part = &line[semi_pos + 1..];
            self.highlight_register_changes(job, reg_part, font_id);
        } else {
            // 只有指令，没有寄存器变化
            let inst_part = &line[pos..];
            self.highlight_instruction_text(job, inst_part, font_id, inst.inst_type);
        }
    }

    fn highlight_instruction_text(
        &self,
        job: &mut egui::text::LayoutJob,
        text: &str,
        font_id: &egui::FontId,
        inst_type: InstructionType,
    ) {
        let inst_color = self.theme.get_instruction_color(inst_type);
        
        // 简单处理：找到指令助记符（第一个非空白 token）
        let trimmed = text.trim_start();
        let leading_space = text.len() - text.trim_start().len();
        
        if leading_space > 0 {
            job.append(&text[..leading_space], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(171, 178, 191),
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
            
            // 操作数（高亮寄存器）
            self.highlight_operands(job, operands, font_id);
        } else {
            // 只有助记符
            job.append(trimmed, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: inst_color,
                ..Default::default()
            });
        }
    }

    fn highlight_operands(&self, job: &mut egui::text::LayoutJob, text: &str, font_id: &egui::FontId) {
        let default_color = egui::Color32::from_rgb(171, 178, 191);
        let mut chars = text.char_indices().peekable();
        let mut last_end = 0;

        while let Some((i, c)) = chars.next() {
            // 检查是否是寄存器开始
            if c == 'x' || c == 'X' || c == 'w' || c == 'W' || c == 's' || c == 'S' {
                // 检查后面是否是数字
                if let Some(&(_, next_c)) = chars.peek() {
                    if next_c.is_ascii_digit() {
                        // 输出前面的部分
                        if i > last_end {
                            job.append(&text[last_end..i], 0.0, egui::TextFormat {
                                font_id: font_id.clone(),
                                color: default_color,
                                ..Default::default()
                            });
                        }
                        
                        // 找到寄存器结束位置
                        let mut reg_end = i + 1;
                        while let Some(&(j, c)) = chars.peek() {
                            if c.is_ascii_digit() {
                                reg_end = j + 1;
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        
                        // 输出寄存器
                        job.append(&text[i..reg_end], 0.0, egui::TextFormat {
                            font_id: font_id.clone(),
                            color: self.theme.register,
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
                    // 检查是否是完整的寄存器名
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
                            color: self.theme.register,
                            ..Default::default()
                        });
                        
                        // 跳过这些字符
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
                
                // 找立即数结束
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
                    color: self.theme.hex_value,
                    ..Default::default()
                });
                last_end = imm_end;
            }
        }
        
        // 输出剩余部分
        if last_end < text.len() {
            job.append(&text[last_end..], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: default_color,
                ..Default::default()
            });
        }
    }

    fn highlight_register_changes(&self, job: &mut egui::text::LayoutJob, text: &str, font_id: &egui::FontId) {
        // 格式: X16=0x0->0x7e8897c000
        let mut pos = 0;
        let _chars: Vec<char> = text.chars().collect(); // 保留用于将来的高亮优化
        
        while pos < text.len() {
            // 找寄存器名
            if let Some(eq_pos) = text[pos..].find('=') {
                let eq_pos = pos + eq_pos;
                let reg_name = &text[pos..eq_pos];
                
                job.append(reg_name, 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: self.theme.register,
                    ..Default::default()
                });
                
                job.append("=", 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(150, 150, 150),
                    ..Default::default()
                });
                
                pos = eq_pos + 1;
                
                // 找 ->
                if let Some(arrow_pos) = text[pos..].find("->") {
                    let arrow_pos = pos + arrow_pos;
                    let old_val = &text[pos..arrow_pos];
                    
                    job.append(old_val, 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: self.theme.register_value,
                        ..Default::default()
                    });
                    
                    job.append("->", 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: egui::Color32::from_rgb(255, 100, 100),
                        ..Default::default()
                    });
                    
                    pos = arrow_pos + 2;
                    
                    // 新值（到空格或结束）
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
                // 没有更多寄存器变化
                job.append(&text[pos..], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
                break;
            }
        }
    }

    fn highlight_memory_line(&self, job: &mut egui::text::LayoutJob, line: &str, font_id: &egui::FontId) {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        
        // 缩进
        if indent > 0 {
            job.append(&line[..indent], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(171, 178, 191),
                ..Default::default()
            });
        }
        
        // MEM_read 或 MEM_write
        let (op_color, op_end) = if trimmed.starts_with("MEM_read") {
            (self.theme.memory_read, indent + 8)
        } else if trimmed.starts_with("MEM_write") {
            (self.theme.memory_write, indent + 9)
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
        
        // 其余部分
        let remaining = &line[op_end..];
        
        // 高亮地址和值
        let mut pos = 0;
        while pos < remaining.len() {
            if let Some(at_pos) = remaining[pos..].find('@') {
                // @ 之前
                job.append(&remaining[pos..pos + at_pos], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
                
                // @ 符号
                job.append("@", 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(150, 150, 150),
                    ..Default::default()
                });
                
                pos = pos + at_pos + 1;
                
                // 地址
                if remaining[pos..].starts_with("0x") {
                    let addr_end = find_hex_end(&remaining[pos..]) + pos;
                    job.append(&remaining[pos..addr_end], 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: self.theme.address,
                        ..Default::default()
                    });
                    pos = addr_end;
                }
            } else if let Some(val_pos) = remaining[pos..].find("val=") {
                // val= 之前
                job.append(&remaining[pos..pos + val_pos], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
                
                job.append("val=", 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(150, 150, 150),
                    ..Default::default()
                });
                
                pos = pos + val_pos + 4;
                
                // 值
                let val_end = remaining[pos..].find(char::is_whitespace).map(|i| pos + i).unwrap_or(remaining.len());
                job.append(&remaining[pos..val_end], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: self.theme.hex_value,
                    ..Default::default()
                });
                pos = val_end;
            } else {
                // 其余部分
                job.append(&remaining[pos..], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::from_rgb(171, 178, 191),
                    ..Default::default()
                });
                break;
            }
        }
    }

    fn highlight_src_reg_line(&self, job: &mut egui::text::LayoutJob, line: &str, font_id: &egui::FontId) {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        
        // 缩进
        if indent > 0 {
            job.append(&line[..indent], 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::from_rgb(171, 178, 191),
                ..Default::default()
            });
        }
        
        // SRC_REG=
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
            
            // 寄存器名
            let after_eq = &trimmed[eq_pos + 1..];
            if let Some(space_pos) = after_eq.find(' ') {
                let reg_name = &after_eq[..space_pos];
                job.append(reg_name, 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: self.theme.register,
                    ..Default::default()
                });
                
                // 剩余部分
                let rest = &after_eq[space_pos..];
                if let Some(val_pos) = rest.find("val=") {
                    job.append(&rest[..val_pos], 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: egui::Color32::from_rgb(171, 178, 191),
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
                        color: self.theme.hex_value,
                        ..Default::default()
                    });
                } else {
                    job.append(rest, 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: egui::Color32::from_rgb(171, 178, 191),
                        ..Default::default()
                    });
                }
            }
        }
    }
}

/// 渲染过滤器面板
pub fn render_filter_panel(
    ui: &mut egui::Ui,
    state: &mut TraceModeState,
    lang: Language,
) {
    let _panel_bg = egui::Color32::from_rgb(30, 30, 35); // 保留用于将来的主题
    let section_bg = egui::Color32::from_rgb(40, 42, 48);
    let accent_color = egui::Color32::from_rgb(86, 182, 194);
    
    ui.vertical(|ui| {
        ui.add_space(8.0);
        
        // 标题
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("🔍").size(16.0));
            ui.label(egui::RichText::new(if lang == Language::Chinese { "Trace 过滤器" } else { "Trace Filter" })
                .size(14.0)
                .strong()
                .color(egui::Color32::WHITE));
        });
        
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);
        
        // 指令类型过滤
        egui::Frame::none()
            .fill(section_bg)
            .rounding(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(if lang == Language::Chinese { "📋 指令类型" } else { "📋 Instruction Type" })
                    .size(12.0)
                    .strong()
                    .color(accent_color));
                ui.add_space(6.0);
                
                let types = [
                    (InstructionType::Arithmetic, "[A]", "算术", "Arithmetic", state.theme.arithmetic),
                    (InstructionType::Logic, "[L]", "逻辑", "Logic", state.theme.logic),
                    (InstructionType::Memory, "[M]", "内存", "Memory", state.theme.memory),
                    (InstructionType::Branch, "[B]", "分支", "Branch", state.theme.branch),
                    (InstructionType::Call, "[C]", "调用", "Call", state.theme.call),
                    (InstructionType::Return, "[R]", "返回", "Return", state.theme.return_),
                ];
                
                ui.columns(2, |cols| {
                    for (i, (inst_type, tag, name_cn, name_en, color)) in types.iter().enumerate() {
                        let col = &mut cols[i % 2];
                        let enabled = state.filter.enabled_types.get(inst_type).copied().unwrap_or(true);
                        let name = if lang == Language::Chinese { *name_cn } else { *name_en };
                        
                        col.horizontal(|ui| {
                            let mut checked = enabled;
                            if ui.checkbox(&mut checked, "").changed() {
                                state.filter.enabled_types.insert(*inst_type, checked);
                                state.invalidate_cache();
                            }
                            ui.label(egui::RichText::new(*tag).color(*color).monospace().size(11.0));
                            ui.label(egui::RichText::new(name).size(11.0).color(egui::Color32::from_rgb(180, 180, 180)));
                        });
                    }
                });
            });
        
        ui.add_space(8.0);
        
        // 深度过滤
        egui::Frame::none()
            .fill(section_bg)
            .rounding(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.filter.depth_filter_enabled, "");
                    ui.label(egui::RichText::new(if lang == Language::Chinese { "🏷️ 调用深度" } else { "🏷️ Call Depth" })
                        .size(12.0)
                        .strong()
                        .color(accent_color));
                });
                
                if state.filter.depth_filter_enabled {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("[D").color(state.theme.depth).monospace());
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut state.depth_input)
                                .desired_width(40.0)
                                .font(egui::FontId::monospace(12.0))
                        );
                        ui.label(egui::RichText::new("]").color(state.theme.depth).monospace());
                        
                        if response.changed() {
                            state.apply_filter_settings();
                        }
                    });
                }
            });
        
        ui.add_space(8.0);
        
        // 寄存器过滤
        egui::Frame::none()
            .fill(section_bg)
            .rounding(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.filter.register_filter_enabled, "");
                    ui.label(egui::RichText::new(if lang == Language::Chinese { "📝 寄存器追踪" } else { "📝 Register Track" })
                        .size(12.0)
                        .strong()
                        .color(accent_color));
                });
                
                if state.filter.register_filter_enabled {
                    ui.add_space(4.0);
                    
                    // 常用寄存器快捷按钮
                    ui.horizontal_wrapped(|ui| {
                        for reg in &["X0", "X1", "X8", "SP", "LR"] {
                            if ui.small_button(egui::RichText::new(*reg).monospace().size(10.0)).clicked() {
                                state.register_input = reg.to_string();
                                state.filter.register_filter = Some(reg.to_string());
                                state.invalidate_cache();
                            }
                        }
                    });
                    
                    ui.add_space(4.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut state.register_input)
                            .hint_text("X0, X1, SP...")
                            .desired_width(ui.available_width() - 8.0)
                            .font(egui::FontId::monospace(12.0))
                    );
                    
                    if response.changed() {
                        state.apply_filter_settings();
                    }
                }
            });
        
        ui.add_space(8.0);
        
        // 序号范围
        egui::Frame::none()
            .fill(section_bg)
            .rounding(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.filter.seq_filter_enabled, "");
                    ui.label(egui::RichText::new(if lang == Language::Chinese { "🔢 序号范围" } else { "🔢 Sequence Range" })
                        .size(12.0)
                        .strong()
                        .color(accent_color));
                });
                
                if state.filter.seq_filter_enabled {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("#").color(state.theme.seq_number).monospace());
                        let r1 = ui.add(
                            egui::TextEdit::singleline(&mut state.seq_start_input)
                                .hint_text("start")
                                .desired_width(60.0)
                                .font(egui::FontId::monospace(11.0))
                        );
                        ui.label(" - ");
                        let r2 = ui.add(
                            egui::TextEdit::singleline(&mut state.seq_end_input)
                                .hint_text("end")
                                .desired_width(60.0)
                                .font(egui::FontId::monospace(11.0))
                        );
                        
                        if r1.changed() || r2.changed() {
                            state.apply_filter_settings();
                        }
                    });
                }
            });
        
        ui.add_space(8.0);
        
        // 地址范围
        egui::Frame::none()
            .fill(section_bg)
            .rounding(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.filter.address_filter_enabled, "");
                    ui.label(egui::RichText::new(if lang == Language::Chinese { "📍 地址范围" } else { "📍 Address Range" })
                        .size(12.0)
                        .strong()
                        .color(accent_color));
                });
                
                if state.filter.address_filter_enabled {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("0x").color(state.theme.address).monospace());
                        let r1 = ui.add(
                            egui::TextEdit::singleline(&mut state.address_start_input)
                                .hint_text("start")
                                .desired_width(80.0)
                                .font(egui::FontId::monospace(11.0))
                        );
                        ui.label(" - 0x");
                        let r2 = ui.add(
                            egui::TextEdit::singleline(&mut state.address_end_input)
                                .hint_text("end")
                                .desired_width(80.0)
                                .font(egui::FontId::monospace(11.0))
                        );
                        
                        if r1.changed() || r2.changed() {
                            state.apply_filter_settings();
                        }
                    });
                }
            });
        
        ui.add_space(8.0);
        
        // 特殊过滤选项
        egui::Frame::none()
            .fill(section_bg)
            .rounding(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(if lang == Language::Chinese { "⚙️ 其他选项" } else { "⚙️ Options" })
                    .size(12.0)
                    .strong()
                    .color(accent_color));
                ui.add_space(4.0);
                
                if ui.checkbox(
                    &mut state.filter.show_memory_only,
                    if lang == Language::Chinese { "仅显示内存操作" } else { "Memory ops only" }
                ).changed() {
                    state.invalidate_cache();
                }
                
                if ui.checkbox(
                    &mut state.filter.show_reg_changes_only,
                    if lang == Language::Chinese { "仅显示寄存器变化" } else { "Reg changes only" }
                ).changed() {
                    state.invalidate_cache();
                }
            });
        
        ui.add_space(16.0);
        
        // 重置按钮
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if ui.add(
                egui::Button::new(egui::RichText::new(if lang == Language::Chinese { "🔄 重置过滤器" } else { "🔄 Reset Filter" }).size(12.0))
                    .fill(egui::Color32::from_rgb(70, 70, 80))
                    .rounding(4.0)
            ).clicked() {
                state.reset_filter();
            }
        });
    });
}

/// 渲染统计面板
pub fn render_stats_panel(
    ui: &mut egui::Ui,
    state: &TraceModeState,
    lang: Language,
) {
    let section_bg = egui::Color32::from_rgb(40, 42, 48);
    let accent_color = egui::Color32::from_rgb(152, 195, 121);
    
    if let Some(ref stats) = state.statistics {
        ui.add_space(4.0);
        
        // 总览
        egui::Frame::none()
            .fill(section_bg)
            .rounding(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📊")
                        .size(14.0));
                    ui.label(egui::RichText::new(if lang == Language::Chinese { "统计信息" } else { "Statistics" })
                        .size(13.0)
                        .strong()
                        .color(accent_color));
                });
                
                ui.add_space(6.0);
                
                ui.horizontal_wrapped(|ui| {
                    // 总指令数
                    ui.label(egui::RichText::new(format!(
                        "{}: {}",
                        if lang == Language::Chinese { "总指令" } else { "Total" },
                        format_number(stats.total_instructions)
                    )).size(11.0).color(egui::Color32::from_rgb(200, 200, 200)));
                    
                    ui.separator();
                    
                    // 最大深度
                    ui.label(egui::RichText::new(format!(
                        "{}: D{}",
                        if lang == Language::Chinese { "最大深度" } else { "Max Depth" },
                        stats.max_depth
                    )).size(11.0).color(state.theme.depth));
                    
                    ui.separator();
                    
                    // 内存操作
                    ui.label(egui::RichText::new(format!(
                        "R:{} W:{}",
                        format_number(stats.memory_reads),
                        format_number(stats.memory_writes)
                    )).size(11.0).color(state.theme.memory));
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
                            let color = state.theme.get_instruction_color(*inst_type);
                            ui.label(egui::RichText::new(format!(
                                "{}{:.0}%",
                                inst_type.tag(),
                                pct
                            )).size(10.0).monospace().color(color));
                        }
                    }
                });
            });
    }
}

/// 解析十六进制地址
fn parse_hex_address(s: &str) -> Option<u64> {
    let s = s.trim();
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    u64::from_str_radix(s, 16).ok()
}

/// 找到十六进制数结束位置
fn find_hex_end(s: &str) -> usize {
    let mut end = 0;
    let chars: Vec<char> = s.chars().collect();
    
    // 跳过 0x
    if s.starts_with("0x") || s.starts_with("0X") {
        end = 2;
    }
    
    while end < chars.len() && chars[end].is_ascii_hexdigit() {
        end += 1;
    }
    
    end
}

/// 格式化数字（添加千位分隔符）
fn format_number(n: usize) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{:.1}K", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

