/// 国际化语言支持模块
/// Internationalization (i18n) language support module

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Chinese,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }

    pub fn all() -> &'static [Language] {
        &[Language::English, Language::Chinese]
    }
}

/// 语言文本结构 / Language text structure
pub struct I18n {
    pub lang: Language,
}

impl Default for I18n {
    fn default() -> Self {
        Self {
            lang: Language::Chinese, // 默认中文
        }
    }
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    pub fn set_language(&mut self, lang: Language) {
        self.lang = lang;
    }

    // ===== 窗口标题 =====
    pub fn window_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Large Text Viewer",
            Language::Chinese => "大文本查看器",
        }
    }

    pub fn window_title_unsaved(&self) -> &'static str {
        match self.lang {
            Language::English => "Large Text Viewer *",
            Language::Chinese => "大文本查看器 *",
        }
    }

    // ===== 菜单栏 =====
    pub fn menu_file(&self) -> &'static str {
        match self.lang {
            Language::English => "File",
            Language::Chinese => "文件",
        }
    }

    pub fn menu_open(&self) -> &'static str {
        match self.lang {
            Language::English => "Open...",
            Language::Chinese => "打开...",
        }
    }

    pub fn menu_save(&self) -> &'static str {
        match self.lang {
            Language::English => "Save (Ctrl+S)",
            Language::Chinese => "保存 (Ctrl+S)",
        }
    }

    pub fn menu_file_info(&self) -> &'static str {
        match self.lang {
            Language::English => "File Info",
            Language::Chinese => "文件信息",
        }
    }

    pub fn menu_exit(&self) -> &'static str {
        match self.lang {
            Language::English => "Exit",
            Language::Chinese => "退出",
        }
    }

    pub fn menu_view(&self) -> &'static str {
        match self.lang {
            Language::English => "View",
            Language::Chinese => "视图",
        }
    }

    pub fn menu_word_wrap(&self) -> &'static str {
        match self.lang {
            Language::English => "Word Wrap",
            Language::Chinese => "自动换行",
        }
    }

    pub fn menu_line_numbers(&self) -> &'static str {
        match self.lang {
            Language::English => "Line Numbers",
            Language::Chinese => "显示行号",
        }
    }

    pub fn menu_dark_mode(&self) -> &'static str {
        match self.lang {
            Language::English => "Dark Mode",
            Language::Chinese => "深色模式",
        }
    }

    pub fn menu_font_size(&self) -> &'static str {
        match self.lang {
            Language::English => "Font Size:",
            Language::Chinese => "字体大小:",
        }
    }

    pub fn menu_select_encoding(&self) -> &'static str {
        match self.lang {
            Language::English => "Select Encoding",
            Language::Chinese => "选择编码",
        }
    }

    pub fn menu_search(&self) -> &'static str {
        match self.lang {
            Language::English => "Search",
            Language::Chinese => "搜索",
        }
    }

    pub fn menu_find(&self) -> &'static str {
        match self.lang {
            Language::English => "Find",
            Language::Chinese => "查找",
        }
    }

    pub fn menu_replace(&self) -> &'static str {
        match self.lang {
            Language::English => "Replace",
            Language::Chinese => "替换",
        }
    }

    pub fn menu_use_regex(&self) -> &'static str {
        match self.lang {
            Language::English => "Use Regex",
            Language::Chinese => "使用正则表达式",
        }
    }

    pub fn menu_match_case(&self) -> &'static str {
        match self.lang {
            Language::English => "Match Case",
            Language::Chinese => "区分大小写",
        }
    }

    pub fn menu_tools(&self) -> &'static str {
        match self.lang {
            Language::English => "Tools",
            Language::Chinese => "工具",
        }
    }

    pub fn menu_tail_mode(&self) -> &'static str {
        match self.lang {
            Language::English => "Tail Mode (Auto-refresh)",
            Language::Chinese => "尾随模式 (自动刷新)",
        }
    }

    pub fn menu_language(&self) -> &'static str {
        match self.lang {
            Language::English => "Language",
            Language::Chinese => "语言",
        }
    }

    // ===== 工具栏 =====
    pub fn toolbar_search(&self) -> &'static str {
        match self.lang {
            Language::English => "Search:",
            Language::Chinese => "搜索:",
        }
    }

    pub fn toolbar_match_case(&self) -> &'static str {
        match self.lang {
            Language::English => "Match Case",
            Language::Chinese => "区分大小写",
        }
    }

    pub fn toolbar_use_regex(&self) -> &'static str {
        match self.lang {
            Language::English => "Use Regex",
            Language::Chinese => "正则表达式",
        }
    }

    pub fn toolbar_find(&self) -> &'static str {
        match self.lang {
            Language::English => "🔍 Find",
            Language::Chinese => "🔍 查找",
        }
    }

    pub fn toolbar_find_all(&self) -> &'static str {
        match self.lang {
            Language::English => "🔎 Find All",
            Language::Chinese => "🔎 查找全部",
        }
    }

    pub fn toolbar_previous(&self) -> &'static str {
        match self.lang {
            Language::English => "⬆ Previous",
            Language::Chinese => "⬆ 上一个",
        }
    }

    pub fn toolbar_next(&self) -> &'static str {
        match self.lang {
            Language::English => "⬇ Next",
            Language::Chinese => "⬇ 下一个",
        }
    }

    pub fn toolbar_searching(&self) -> &'static str {
        match self.lang {
            Language::English => "Searching...",
            Language::Chinese => "搜索中...",
        }
    }

    pub fn toolbar_stop(&self) -> &'static str {
        match self.lang {
            Language::English => "Stop",
            Language::Chinese => "停止",
        }
    }

    pub fn toolbar_goto_line(&self) -> &'static str {
        match self.lang {
            Language::English => "Go to line:",
            Language::Chinese => "跳转到行:",
        }
    }

    pub fn toolbar_go(&self) -> &'static str {
        match self.lang {
            Language::English => "Go",
            Language::Chinese => "跳转",
        }
    }

    pub fn toolbar_replace_with(&self) -> &'static str {
        match self.lang {
            Language::English => "Replace with:",
            Language::Chinese => "替换为:",
        }
    }

    pub fn toolbar_replacement_text(&self) -> &'static str {
        match self.lang {
            Language::English => "Replacement text...",
            Language::Chinese => "替换文本...",
        }
    }

    pub fn toolbar_stop_replace(&self) -> &'static str {
        match self.lang {
            Language::English => "Stop Replace",
            Language::Chinese => "停止替换",
        }
    }

    pub fn toolbar_replace(&self) -> &'static str {
        match self.lang {
            Language::English => "Replace",
            Language::Chinese => "替换",
        }
    }

    pub fn toolbar_replace_all(&self) -> &'static str {
        match self.lang {
            Language::English => "Replace All",
            Language::Chinese => "全部替换",
        }
    }

    // ===== 状态栏 =====
    pub fn status_file(&self) -> &'static str {
        match self.lang {
            Language::English => "File:",
            Language::Chinese => "文件:",
        }
    }

    pub fn status_size(&self) -> &'static str {
        match self.lang {
            Language::English => "Size:",
            Language::Chinese => "大小:",
        }
    }

    pub fn status_bytes(&self) -> &'static str {
        match self.lang {
            Language::English => "bytes",
            Language::Chinese => "字节",
        }
    }

    pub fn status_lines(&self) -> &'static str {
        match self.lang {
            Language::English => "Lines:",
            Language::Chinese => "行数:",
        }
    }

    pub fn status_encoding(&self) -> &'static str {
        match self.lang {
            Language::English => "Encoding:",
            Language::Chinese => "编码:",
        }
    }

    pub fn status_line(&self) -> &'static str {
        match self.lang {
            Language::English => "Line:",
            Language::Chinese => "行:",
        }
    }

    pub fn status_no_file(&self) -> &'static str {
        match self.lang {
            Language::English => "No file opened - Click File → Open to start",
            Language::Chinese => "未打开文件 - 点击 文件 → 打开 开始使用",
        }
    }

    // ===== 主界面 =====
    pub fn main_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Large Text Viewer",
            Language::Chinese => "大文本查看器",
        }
    }

    pub fn main_hint(&self) -> &'static str {
        match self.lang {
            Language::English => "\nClick File → Open to load a text file",
            Language::Chinese => "\n点击 文件 → 打开 加载文本文件",
        }
    }

    // ===== 对话框 =====
    pub fn dialog_select_encoding(&self) -> &'static str {
        match self.lang {
            Language::English => "Select Encoding",
            Language::Chinese => "选择编码",
        }
    }

    pub fn dialog_cancel(&self) -> &'static str {
        match self.lang {
            Language::English => "Cancel",
            Language::Chinese => "取消",
        }
    }

    pub fn dialog_file_info(&self) -> &'static str {
        match self.lang {
            Language::English => "File Information",
            Language::Chinese => "文件信息",
        }
    }

    pub fn dialog_path(&self) -> &'static str {
        match self.lang {
            Language::English => "Path:",
            Language::Chinese => "路径:",
        }
    }

    pub fn dialog_close(&self) -> &'static str {
        match self.lang {
            Language::English => "Close",
            Language::Chinese => "关闭",
        }
    }

    // ===== 状态消息 =====
    pub fn msg_opened(&self, path: &str) -> String {
        match self.lang {
            Language::English => format!("Opened: {}", path),
            Language::Chinese => format!("已打开: {}", path),
        }
    }

    pub fn msg_error_opening(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Error opening file: {}", error),
            Language::Chinese => format!("打开文件出错: {}", error),
        }
    }

    pub fn msg_search_running(&self) -> &'static str {
        match self.lang {
            Language::English => "Search already running...",
            Language::Chinese => "搜索已在进行中...",
        }
    }

    pub fn msg_open_file_first(&self) -> &'static str {
        match self.lang {
            Language::English => "Open a file before searching",
            Language::Chinese => "请先打开一个文件再搜索",
        }
    }

    pub fn msg_enter_query(&self) -> &'static str {
        match self.lang {
            Language::English => "Enter a search query first",
            Language::Chinese => "请先输入搜索内容",
        }
    }

    pub fn msg_searching_all(&self) -> &'static str {
        match self.lang {
            Language::English => "Searching all matches...",
            Language::Chinese => "正在搜索所有匹配项...",
        }
    }

    pub fn msg_searching_first(&self) -> &'static str {
        match self.lang {
            Language::English => "Searching first match...",
            Language::Chinese => "正在搜索第一个匹配项...",
        }
    }

    pub fn msg_found_matches(&self, count: usize) -> String {
        match self.lang {
            Language::English => format!("Found {} matches...", count),
            Language::Chinese => format!("已找到 {} 个匹配项...", count),
        }
    }

    pub fn msg_search_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Search failed: {}", error),
            Language::Chinese => format!("搜索失败: {}", error),
        }
    }

    pub fn msg_found_total(&self, count: usize) -> String {
        match self.lang {
            Language::English => format!("Found {} matches", count),
            Language::Chinese => format!("共找到 {} 个匹配项", count),
        }
    }

    pub fn msg_first_match_hint(&self) -> &'static str {
        match self.lang {
            Language::English => "Showing first match. Run Find All to see every result.",
            Language::Chinese => "显示第一个匹配项。点击「查找全部」查看所有结果。",
        }
    }

    pub fn msg_no_matches(&self) -> &'static str {
        match self.lang {
            Language::English => "No matches found",
            Language::Chinese => "未找到匹配项",
        }
    }

    pub fn msg_search_stopped(&self) -> &'static str {
        match self.lang {
            Language::English => "Search stopped by user",
            Language::Chinese => "搜索已被用户停止",
        }
    }

    pub fn msg_replacing(&self, progress: f32) -> String {
        match self.lang {
            Language::English => format!("Replacing... {:.1}%", progress * 100.0),
            Language::Chinese => format!("替换中... {:.1}%", progress * 100.0),
        }
    }

    pub fn msg_replace_complete(&self) -> &'static str {
        match self.lang {
            Language::English => "Replacement complete.",
            Language::Chinese => "替换完成。",
        }
    }

    pub fn msg_replace_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Replace failed: {}", error),
            Language::Chinese => format!("替换失败: {}", error),
        }
    }

    pub fn msg_pending_save(&self) -> &'static str {
        match self.lang {
            Language::English => "Replacement pending. Save to apply changes.",
            Language::Chinese => "替换已挂起。请保存以应用更改。",
        }
    }

    pub fn msg_error_saving(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Error saving: {}", error),
            Language::Chinese => format!("保存出错: {}", error),
        }
    }

    pub fn msg_saved(&self) -> &'static str {
        match self.lang {
            Language::English => "File saved successfully",
            Language::Chinese => "文件保存成功",
        }
    }

    pub fn msg_error_reopening(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Error re-opening file: {}", error),
            Language::Chinese => format!("重新打开文件出错: {}", error),
        }
    }

    pub fn msg_error_copy(&self) -> &'static str {
        match self.lang {
            Language::English => "Error copying file for save",
            Language::Chinese => "复制文件用于保存时出错",
        }
    }

    pub fn msg_loading_results(&self, start: usize, end: usize) -> String {
        match self.lang {
            Language::English => format!("Loading results {}...{}", start + 1, end),
            Language::Chinese => format!("正在加载结果 {}...{}", start + 1, end),
        }
    }

    pub fn msg_jumped_to_line(&self, line: usize) -> String {
        match self.lang {
            Language::English => format!("Jumped to line {}", line),
            Language::Chinese => format!("已跳转到第 {} 行", line),
        }
    }

    pub fn msg_line_out_of_range(&self) -> &'static str {
        match self.lang {
            Language::English => "Line number out of range",
            Language::Chinese => "行号超出范围",
        }
    }

    pub fn msg_invalid_line(&self) -> &'static str {
        match self.lang {
            Language::English => "Invalid line number",
            Language::Chinese => "无效的行号",
        }
    }

    pub fn msg_cannot_wrap(&self) -> &'static str {
        match self.lang {
            Language::English => "Cannot wrap to end in paginated mode yet.",
            Language::Chinese => "分页模式下暂不支持循环到末尾。",
        }
    }

    pub fn msg_counted_in(&self, elapsed: &str) -> String {
        match self.lang {
            Language::English => format!("(Counted in {})", elapsed),
            Language::Chinese => format!("(统计耗时 {})", elapsed),
        }
    }

    pub fn msg_rendered_in(&self, elapsed: &str) -> String {
        match self.lang {
            Language::English => format!("(Rendered in {})", elapsed),
            Language::Chinese => format!("(渲染耗时 {})", elapsed),
        }
    }

    pub fn search_error_prefix(&self) -> &'static str {
        match self.lang {
            Language::English => "Search error:",
            Language::Chinese => "搜索错误:",
        }
    }
}

