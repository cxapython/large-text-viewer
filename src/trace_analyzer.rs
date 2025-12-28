//! Arm64Trace 文件分析器
//! 
//! 解析 QBDI Trace v2.0 格式的 trace 文件，提供：
//! - 指令类型识别和统计
//! - 寄存器变化追踪
//! - 内存访问分析
//! - 调用深度过滤

use std::collections::HashMap;
use regex::Regex;

/// 指令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionType {
    Arithmetic,  // [A] 算术指令
    Logic,       // [L] 逻辑指令
    Memory,      // [M] 内存指令
    Branch,      // [B] 分支指令
    Call,        // [C] 调用指令
    Return,      // [R] 返回指令
    Unknown,     // 未知类型
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

    pub fn name(&self) -> &'static str {
        match self {
            Self::Arithmetic => "算术",
            Self::Logic => "逻辑",
            Self::Memory => "内存",
            Self::Branch => "分支",
            Self::Call => "调用",
            Self::Return => "返回",
            Self::Unknown => "未知",
        }
    }

    pub fn name_en(&self) -> &'static str {
        match self {
            Self::Arithmetic => "Arithmetic",
            Self::Logic => "Logic",
            Self::Memory => "Memory",
            Self::Branch => "Branch",
            Self::Call => "Call",
            Self::Return => "Return",
            Self::Unknown => "Unknown",
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

/// 内存操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOpType {
    Read,
    Write,
}

/// 解析后的指令信息
#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub seq_num: u64,              // 序号
    pub depth: u8,                 // 调用深度
    pub inst_type: InstructionType, // 指令类型
    pub address: u64,              // 虚拟地址
    pub offset: u64,               // 模块偏移
    pub disasm: String,            // 反汇编文本
    pub reg_changes: Vec<RegisterChange>, // 寄存器变化
    pub memory_ops: Vec<MemoryOp>,  // 内存操作
}

/// 寄存器变化记录
#[derive(Debug, Clone)]
pub struct RegisterChange {
    pub reg_name: String,
    pub old_value: u64,
    pub new_value: u64,
}

/// 内存操作记录
#[derive(Debug, Clone)]
pub struct MemoryOp {
    pub op_type: MemoryOpType,
    pub address: u64,
    pub size: usize,
    pub value: String,
    pub src_reg: Option<String>,
    pub src_value: Option<u64>,
}

/// Trace 统计信息
#[derive(Debug, Clone, Default)]
pub struct TraceStatistics {
    pub total_instructions: usize,
    pub instruction_counts: HashMap<InstructionType, usize>,
    pub max_depth: u8,
    pub memory_reads: usize,
    pub memory_writes: usize,
    pub register_usage: HashMap<String, usize>,
    pub function_entries: usize,
    pub function_exits: usize,
}

/// Trace 过滤器配置
#[derive(Debug, Clone)]
pub struct TraceFilter {
    pub enabled_types: HashMap<InstructionType, bool>,
    pub depth_filter: Option<u8>,
    pub depth_filter_enabled: bool,
    pub register_filter: Option<String>,
    pub register_filter_enabled: bool,
    pub address_range: Option<(u64, u64)>,
    pub address_filter_enabled: bool,
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
            address_range: None,
            address_filter_enabled: false,
            seq_range: None,
            seq_filter_enabled: false,
            show_memory_only: false,
            show_reg_changes_only: false,
        }
    }
}

/// Arm64Trace 分析器
pub struct TraceAnalyzer {
    // 正则表达式（延迟初始化）
    instruction_regex: Option<Regex>,
    reg_change_regex: Option<Regex>,
    mem_op_regex: Option<Regex>,
    src_reg_regex: Option<Regex>,
    enter_regex: Option<Regex>,
    leave_regex: Option<Regex>,
}

impl Default for TraceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceAnalyzer {
    pub fn new() -> Self {
        Self {
            instruction_regex: None,
            reg_change_regex: None,
            mem_op_regex: None,
            src_reg_regex: None,
            enter_regex: None,
            leave_regex: None,
        }
    }

    /// 初始化正则表达式
    fn init_regex(&mut self) {
        if self.instruction_regex.is_none() {
            // 匹配指令行: #1 [D1] [M] 0x0000007dd046e244	0x21244		ldr	x16, #0x8	;X16=0x0->0x7e8897c000
            self.instruction_regex = Regex::new(
                r"^#(\d+)\s+\[D(\d+)\]\s+\[([ALMBC R])\]\s+0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)\s+(.+?)(?:;(.*))?$"
            ).ok();

            // 匹配寄存器变化: X16=0x0->0x7e8897c000
            self.reg_change_regex = Regex::new(
                r"([XWS][0-9]+|SP|LR|PC)=0x([0-9a-fA-F]+)->0x([0-9a-fA-F]+)"
            ).ok();

            // 匹配内存操作: MEM_read @0x7dd046e24c size=8 val=00c097887e000000
            self.mem_op_regex = Regex::new(
                r"MEM_(read|write)\s+@0x([0-9a-fA-F]+)\s+size=(\d+)\s+val=([0-9a-fA-F]+)"
            ).ok();

            // 匹配源寄存器: SRC_REG=X20 val=0x0
            self.src_reg_regex = Regex::new(
                r"SRC_REG=([XWS][0-9]+|SP|LR)\s+val=0x([0-9a-fA-F]+)"
            ).ok();

            // 匹配函数入口: ====== ENTER 0x7dd046e244 (global) ======
            self.enter_regex = Regex::new(
                r"======\s+ENTER\s+0x([0-9a-fA-F]+)"
            ).ok();

            // 匹配函数出口: ======  LEAVE 0x7dd046e244 ======
            self.leave_regex = Regex::new(
                r"======\s+LEAVE\s+0x([0-9a-fA-F]+)"
            ).ok();
        }
    }

    /// 检测文件是否为 Arm64Trace 格式
    pub fn detect_trace_format(content: &str) -> bool {
        let first_lines: String = content.lines().take(50).collect::<Vec<_>>().join("\n");
        
        // 检测特征
        first_lines.contains("QBDI Trace") 
            || first_lines.contains("[hook] target=")
            || (first_lines.contains("[D") && first_lines.contains("] ["))
            || first_lines.contains("====== ENTER")
            || Regex::new(r"#\d+\s+\[D\d+\]\s+\[[ALMBC R]\]").map(|r| r.is_match(&first_lines)).unwrap_or(false)
    }

    /// 解析单行指令
    pub fn parse_instruction(&mut self, line: &str) -> Option<ParsedInstruction> {
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

        // 解析寄存器变化
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
            memory_ops: Vec::new(),
        })
    }

    /// 解析内存操作行
    pub fn parse_memory_op(&mut self, line: &str) -> Option<MemoryOp> {
        self.init_regex();
        
        let regex = self.mem_op_regex.as_ref()?;
        let caps = regex.captures(line)?;

        let op_type = match caps.get(1)?.as_str() {
            "read" => MemoryOpType::Read,
            "write" => MemoryOpType::Write,
            _ => return None,
        };
        let address = u64::from_str_radix(caps.get(2)?.as_str(), 16).ok()?;
        let size = caps.get(3)?.as_str().parse().ok()?;
        let value = caps.get(4)?.as_str().to_string();

        // 检查是否有源寄存器信息
        let (src_reg, src_value) = if let Some(ref src_regex) = self.src_reg_regex {
            if let Some(src_caps) = src_regex.captures(line) {
                let reg = src_caps.get(1).map(|m| m.as_str().to_string());
                let val = src_caps.get(2).and_then(|m| u64::from_str_radix(m.as_str(), 16).ok());
                (reg, val)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        Some(MemoryOp {
            op_type,
            address,
            size,
            value,
            src_reg,
            src_value,
        })
    }

    /// 检查是否为函数入口
    pub fn is_function_enter(&mut self, line: &str) -> Option<u64> {
        self.init_regex();
        
        if let Some(ref regex) = self.enter_regex {
            if let Some(caps) = regex.captures(line) {
                return caps.get(1).and_then(|m| u64::from_str_radix(m.as_str(), 16).ok());
            }
        }
        None
    }

    /// 检查是否为函数出口
    pub fn is_function_leave(&mut self, line: &str) -> Option<u64> {
        self.init_regex();
        
        if let Some(ref regex) = self.leave_regex {
            if let Some(caps) = regex.captures(line) {
                return caps.get(1).and_then(|m| u64::from_str_radix(m.as_str(), 16).ok());
            }
        }
        None
    }

    /// 统计 trace 文件信息（采样统计，避免处理整个大文件）
    pub fn compute_statistics(&mut self, content: &str, sample_size: usize) -> TraceStatistics {
        let mut stats = TraceStatistics::default();
        
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        // 采样统计
        let step = if total_lines > sample_size { total_lines / sample_size } else { 1 };
        
        for (i, line) in lines.iter().enumerate() {
            // 计数所有行中的函数入口/出口
            if self.is_function_enter(line).is_some() {
                stats.function_entries += 1;
            }
            if self.is_function_leave(line).is_some() {
                stats.function_exits += 1;
            }

            // 采样统计指令
            if i % step != 0 && i != 0 && i != total_lines - 1 {
                continue;
            }

            if let Some(inst) = self.parse_instruction(line) {
                stats.total_instructions += 1;
                *stats.instruction_counts.entry(inst.inst_type).or_insert(0) += 1;
                stats.max_depth = stats.max_depth.max(inst.depth);

                for reg in &inst.reg_changes {
                    *stats.register_usage.entry(reg.reg_name.clone()).or_insert(0) += 1;
                }
            }

            // 统计内存操作
            if let Some(mem_op) = self.parse_memory_op(line) {
                match mem_op.op_type {
                    MemoryOpType::Read => stats.memory_reads += 1,
                    MemoryOpType::Write => stats.memory_writes += 1,
                }
            }
        }

        // 如果是采样，估算总数
        if step > 1 {
            let scale = total_lines as f64 / sample_size as f64;
            stats.total_instructions = (stats.total_instructions as f64 * scale) as usize;
            stats.memory_reads = (stats.memory_reads as f64 * scale) as usize;
            stats.memory_writes = (stats.memory_writes as f64 * scale) as usize;
            for count in stats.instruction_counts.values_mut() {
                *count = (*count as f64 * scale) as usize;
            }
        }

        stats
    }

    /// 检查行是否匹配过滤器
    pub fn matches_filter(&mut self, line: &str, filter: &TraceFilter) -> bool {
        // 空行或注释行始终显示
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') && !trimmed.starts_with("#1") {
            return true;
        }

        // 函数入口/出口始终显示
        if trimmed.contains("======") {
            return true;
        }

        // hook 信息始终显示
        if trimmed.starts_with("[hook]") || trimmed.starts_with("[gqb]") {
            return true;
        }

        // 解析指令
        if let Some(inst) = self.parse_instruction(line) {
            // 检查指令类型过滤
            if let Some(&enabled) = filter.enabled_types.get(&inst.inst_type) {
                if !enabled {
                    return false;
                }
            }

            // 检查深度过滤
            if filter.depth_filter_enabled {
                if let Some(depth) = filter.depth_filter {
                    if inst.depth != depth {
                        return false;
                    }
                }
            }

            // 检查序号范围过滤
            if filter.seq_filter_enabled {
                if let Some((start, end)) = filter.seq_range {
                    if inst.seq_num < start || inst.seq_num > end {
                        return false;
                    }
                }
            }

            // 检查地址范围过滤
            if filter.address_filter_enabled {
                if let Some((start, end)) = filter.address_range {
                    if inst.address < start || inst.address > end {
                        return false;
                    }
                }
            }

            // 检查寄存器过滤
            if filter.register_filter_enabled {
                if let Some(ref reg) = filter.register_filter {
                    let has_reg = inst.reg_changes.iter().any(|r| r.reg_name.eq_ignore_ascii_case(reg))
                        || inst.disasm.to_uppercase().contains(&reg.to_uppercase());
                    if !has_reg {
                        return false;
                    }
                }
            }

            // 只显示有寄存器变化的行
            if filter.show_reg_changes_only && inst.reg_changes.is_empty() {
                return false;
            }

            return true;
        }

        // 内存操作行
        if let Some(_mem_op) = self.parse_memory_op(line) {
            // 如果启用了只显示内存操作，这行应该显示
            if filter.show_memory_only {
                return true;
            }
            // 否则跟随上一条指令的可见性（简化处理：显示）
            return true;
        }

        // 其他行（如 SRC_REG 行）跟随显示
        true
    }
}

/// ARM64 寄存器列表
pub const ARM64_REGISTERS: &[&str] = &[
    "X0", "X1", "X2", "X3", "X4", "X5", "X6", "X7",
    "X8", "X9", "X10", "X11", "X12", "X13", "X14", "X15",
    "X16", "X17", "X18", "X19", "X20", "X21", "X22", "X23",
    "X24", "X25", "X26", "X27", "X28", "X29", "X30",
    "SP", "LR", "PC",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instruction() {
        let mut analyzer = TraceAnalyzer::new();
        let line = "#1 [D1] [M] 0x0000007dd046e244\t0x21244\t\tldr\tx16, #0x8\t;X16=0x0->0x7e8897c000";
        
        let inst = analyzer.parse_instruction(line);
        assert!(inst.is_some());
        
        let inst = inst.unwrap();
        assert_eq!(inst.seq_num, 1);
        assert_eq!(inst.depth, 1);
        assert_eq!(inst.inst_type, InstructionType::Memory);
        assert_eq!(inst.address, 0x7dd046e244);
    }

    #[test]
    fn test_detect_trace_format() {
        let content = "# QBDI Trace v2.0\n#1 [D1] [M] 0x1234 0x100 ldr x0, [x1]";
        assert!(TraceAnalyzer::detect_trace_format(content));

        let content = "just some random text\nno trace here";
        assert!(!TraceAnalyzer::detect_trace_format(content));
    }
}

