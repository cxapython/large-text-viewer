# 高性能大文本文件查看器

**基于 Rust 开发的大文本文件查看器**

---

## 项目简介

在系统编程、数据科学和运维领域，开发者经常需要处理海量文本文件——服务器日志、数据库转储和仿真输出——这些文件很容易超过数GB大小。现代代码编辑器如 Visual Studio Code 和 Zed 主要针对代码编辑进行优化（AST 解析、语法高亮），而非原始数据吞吐。因此，尝试打开数GB大小的文件通常会导致编辑器因内存消耗过大而冻结或崩溃。

用户往往被迫使用命令行工具如 `less` 或 `grep`，这些工具虽然高效，但缺乏图形用户界面的交互性和便利性。本项目旨在结合 CLI 工具的高性能和现代 GUI 的可用性，使用 Rust 的零成本抽象和内存安全保证来解决这一实际痛点。

## 功能特性

![界面截图](docs/UI_Screenshot.png)

### 核心功能

1. **内存映射文件查看**：利用 `memmap2` 将文件视为内存切片，让操作系统处理分页。这使得打开比可用 RAM 更大的文件成为可能。

2. **混合行索引**：
   - **全量索引**：对于小文件（<10MB），提供精确的行映射
   - **稀疏索引**：对于大文件，使用检查点系统保持索引内存使用量可忽略不计（100GB文件仅需<1MB）

3. **虚拟滚动**：仅渲染当前视口中可见的行，确保恒定的渲染性能

4. **多线程搜索引擎**：
   - 基于 `regex` 库的高性能正则匹配
   - 支持区分大小写和正则表达式查询
   - 多线程并行搜索，充分利用多核 CPU
   - "查找全部"操作在后台线程运行，不阻塞 UI
   - 实时进度报告和结果流式传输

5. **流式替换**：
   - **就地替换**：针对相同长度字符串的优化替换
   - **写时复制**：使用临时文件进行安全、原子的不同长度字符串替换
   - **挂起替换**："虚拟编辑"允许用户在内存中排队更改，然后再提交到磁盘

6. **编码支持**：自动检测并支持 UTF-8、UTF-16（LE/BE）和 Windows-1252

7. **中英文切换**：支持界面语言在中文和英文之间切换

8. **搜索结果面板**：底部显示搜索结果列表，点击可跳转，类似 010 Editor

9. **快捷键支持**：
   | 快捷键 | 功能 |
   |--------|------|
   | ⌘F / Ctrl+F | 打开搜索 |
   | ⌘R / Ctrl+R | 打开替换 |
   | ⌘S / Ctrl+S | 保存文件 |
   | F3 | 下一个结果 |
   | Shift+F3 | 上一个结果 |
   | Escape | 关闭搜索栏 |

### 🔌 插件系统 (v0.2.0 新增)

应用现在支持可扩展的插件架构，允许添加自定义文件类型处理器：

#### 内置插件：ARM64 Trace 分析器

专为逆向工程设计的 QBDI Trace 文件分析插件：

- **🎨 语法高亮**：
  - 指令类型颜色区分（算术、逻辑、内存、分支、调用、返回）
  - 寄存器名称高亮
  - 地址和十六进制值着色
  - 函数入口/出口标记

- **📊 统计面板**：
  - 总指令数统计
  - 最大调用深度
  - 内存读写次数
  - 指令类型分布百分比

- **🔍 过滤器**：
  - 按指令类型过滤（[A] [L] [M] [B] [C] [R]）
  - 按调用深度过滤
  - 按寄存器追踪（X0, X1, SP, LR 等）
  - 按序号范围过滤

- **自动检测**：打开 QBDI Trace 文件时自动启用

#### 开发自定义插件

实现 `Plugin` trait 即可创建新插件：

```rust
pub trait Plugin: Send {
    /// 返回插件元信息
    fn info(&self) -> &PluginInfo;
    
    /// 检测是否可以处理此文件内容（根据文件头部采样判断）
    fn can_handle(&self, content_sample: &str) -> bool;
    
    /// 插件激活时调用（文件加载时）
    fn on_activate(&mut self, content_sample: &str);
    
    /// 插件停用时调用
    fn on_deactivate(&mut self);
    
    /// 渲染侧边面板（过滤器、统计等）
    fn render_side_panel(&mut self, ui: &mut egui::Ui, ctx: &PluginContext);
    
    /// 渲染单行内容（语法高亮）
    fn render_line(&mut self, ui: &mut egui::Ui, line: &str, ctx: &PluginContext);
}
```

**开发步骤：**

1. 在 `src/plugins/` 目录下创建新的 `.rs` 文件
2. 实现 `Plugin` trait 的所有必需方法
3. 在 `src/main.rs` 中通过 `PluginManager::register()` 注册插件
4. 插件会在文件加载时自动检测并激活

**示例：** 参考 `src/trace_plugin.rs` 的实现

#### Python 插件支持 (可选)

现在支持使用 Python 3 编写插件！启用方法：

```bash
# 编译时启用 Python 插件支持
cargo build --release --features python-plugins
```

**Python 插件目录：** `~/.large-text-viewer/plugins/`

**快速开始：**

```python
# ~/.large-text-viewer/plugins/my_plugin.py
class LTVPlugin:
    def info(self):
        return {
            'id': 'my_plugin',
            'name': 'My Plugin',
            'name_cn': '我的插件',
            'description': 'A custom plugin',
            'description_cn': '自定义插件',
            'version': '1.0.0',
            'author': 'Your Name',
            'icon': '🔧',
        }
    
    def can_handle(self, content_sample):
        return 'MY_MARKER' in content_sample
    
    def on_activate(self, content_sample):
        pass
    
    def on_deactivate(self):
        pass
    
    def get_panel_data(self):
        return {'items': [
            {'type': 'section', 'label': '📊 Stats'},
            {'type': 'stat', 'label': 'Lines', 'value': '100'},
        ]}
    
    def process_line(self, line):
        return [{'text': line}]
```

详细文档请参考 `docs/PLUGIN_DEVELOPMENT.md`

## 性能目标

1. **即时访问**：即时打开 >4GB 的文件（<100ms），无需将全部内容加载到 RAM
2. **高效搜索**：对文字字符串和正则表达式进行多线程搜索，延迟低于一秒
3. **安全编辑**：使用原子替换操作启用内容修改，确保数据完整性
4. **低内存占用**：无论文件大小，保持基础内存使用低于 100MB
5. **响应性**：确保 UI 在繁重后台操作期间保持响应（60 FPS）

## 安装指南

### 方法一：使用 Cargo 安装

```bash
cargo install large-text-viewer
```

安装后，在终端输入以下命令启动：

```bash
large-text-viewer
```

### 方法二：从源码编译

1. 确保已安装 Rust 工具链：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. 克隆仓库并编译：

```bash
git clone https://github.com/acejarvis/large-text-viewer
cd large-text-viewer
cargo build --release
```

3. 运行程序：

```bash
cargo run --release
```

或直接运行编译好的二进制文件：

```bash
./target/release/large-text-viewer
```

## 使用指南

### 1. 打开文件

- 启动应用程序
- 点击菜单栏的 **文件 > 打开...**
- 选择任意文本文件（无大小限制），文件将立即加载
- *注意*：底部状态栏显示文件大小、估计行数和检测到的编码

### 2. 导航

- **滚动**：使用鼠标滚轮或右侧滚动条进行导航
- **跳转到行**：在工具栏的"跳转到行"框中输入行号，按回车或点击"跳转"

### 3. 搜索

- 按 **⌘F** (macOS) 或 **Ctrl+F** (Windows/Linux) 打开搜索工具栏
- 在文本框中输入查询内容
- **选项**：
  - 切换 **Aa** 启用区分大小写
  - 切换 **.\*** 启用正则表达式
- **操作**：
  - 点击 **查找**（或按回车）执行搜索
  - 点击 **查找全部** 统计文件中的所有匹配项
  - 使用 **上一个** / **下一个** 按钮或 **F3** / **Shift+F3** 在结果间导航
- **搜索结果面板**：
  - 底部自动显示搜索结果列表
  - 显示行号和匹配内容，点击可直接跳转
  - 匹配文本黄色高亮显示
  - 可折叠/展开、可调整大小

### 4. 替换

- 按 **⌘R** (macOS) 或 **Ctrl+R** (Windows/Linux) 打开替换工具栏
- 输入搜索词和替换文本
- **单次替换**：点击 **替换** 为当前匹配项排队更改。这是"挂起"更改，不会立即写入磁盘
- **全部替换**：点击 **全部替换**。系统将提示您选择输出文件位置。操作将在后台处理文件并写入新文件

### 5. 保存更改

- 如果您进行了单次替换，窗口标题将显示星号 (*)
- 按 **⌘S** (macOS) 或 **Ctrl+S** (Windows/Linux) 或点击 **文件 > 保存**
- 您可以覆盖当前文件或保存到新路径。保存过程中将应用挂起的替换

### 6. 切换语言

- 点击菜单栏的 **语言**
- 选择 **中文** 或 **English**

### 7. 使用插件（ARM64 Trace 分析器）

- 打开 QBDI Trace 格式的文件，插件将自动激活
- 左侧面板显示统计信息和过滤器
- 可通过 **视图 > Trace 模式** 手动启用/禁用

## 架构设计

### 高层组件

系统分为三个主要层：

1. **核心层 (`large-text-core`)**：
   - **`FileReader`**：使用 `memmap2` crate 管理内存映射文件访问，处理编码检测和解码
   - **`LineIndexer`**：负责将行号映射到字节偏移量，实现混合索引策略以平衡内存使用和访问速度
   - **`SearchEngine`**：基于 `regex` 库的多线程搜索引擎，支持并行搜索和流式结果返回
   - **`Replacer`**：处理文件修改，通过写时复制机制确保数据完整性

2. **插件层**：
   - **`Plugin` trait**：定义插件接口
   - **`PluginManager`**：管理插件的注册、激活和生命周期
   - **`TraceAnalyzerPlugin`**：ARM64 Trace 分析插件实现

3. **UI 层 (`large-text-viewer`)**：
   - 使用 `egui` 构建，这是 Rust 的即时模式 GUI 库
   - **`TextViewerApp`**：主应用程序状态管理器，处理用户输入、管理视口（虚拟滚动）并通过通道协调异步任务

### 数据流

应用程序大量依赖异步通信来保持 UI 响应。

- **搜索**：UI 向 `SearchEngine` 发送查询，后者生成线程。结果通过 `mpsc` 通道流式返回 UI，允许 UI 实时更新进度条和高亮匹配项，而不阻塞渲染循环

- **渲染**：UI 仅请求当前视口中可见的行。`LineIndexer` 计算字节范围，`FileReader` 仅从内存映射解码那些特定字节

- **插件渲染**：激活的插件接管行内容渲染，提供语法高亮和专业的可视化效果

## 生成测试文件

要测试大文件功能，可以使用提供的脚本生成大型测试文件（如 1GB）：

```bash
# 使脚本可执行
chmod +x scripts/generate_test_files.sh
# 运行脚本（在 'test_files' 目录中生成文件）
./scripts/generate_test_files.sh
```

或者直接运行单元测试：

```bash
cargo test --workspace
```

## 许可证

MIT License

## 相关链接

- [GitHub 仓库](https://github.com/acejarvis/large-text-viewer)
- [Crates.io - large-text-viewer](https://crates.io/crates/large-text-viewer)
- [Crates.io - large-text-core](https://crates.io/crates/large-text-core)
