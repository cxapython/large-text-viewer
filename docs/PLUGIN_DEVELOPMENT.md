# Large Text Viewer 插件开发指南

本文档详细介绍如何为 Large Text Viewer 开发插件，支持 Rust 原生插件和 Python 3 插件两种方式。

## 目录

- [概述](#概述)
- [快速开始](#快速开始)
- [Python 插件开发](#python-插件开发)
  - [安装和配置](#安装和配置)
  - [插件结构](#插件结构)
  - [API 参考](#api-参考)
  - [示例插件](#示例插件)
- [Rust 插件开发](#rust-插件开发)
  - [Plugin Trait](#plugin-trait)
  - [内置插件示例](#内置插件示例)
- [最佳实践](#最佳实践)
- [常见问题](#常见问题)

## 概述

Large Text Viewer 提供了灵活的插件系统，允许开发者扩展应用功能：

- **语法高亮**: 为特定文件类型提供自定义语法高亮
- **侧边面板**: 显示文件分析数据、统计信息、过滤器等
- **文件检测**: 自动识别文件类型并激活对应插件
- **交互操作**: 通过按钮和动作与用户交互

### 插件类型

| 类型 | 语言 | 性能 | 开发难度 | 适用场景 |
|------|------|------|----------|----------|
| 原生插件 | Rust | ⭐⭐⭐⭐⭐ | 较高 | 性能关键、深度集成 |
| Python 插件 | Python 3 | ⭐⭐⭐ | 简单 | 快速开发、原型验证 |

## 快速开始

### Python 插件（推荐新手）

1. **创建插件目录**（如果不存在）:

```bash
mkdir -p ~/.large-text-viewer/plugins
```

2. **创建插件文件** `~/.large-text-viewer/plugins/my_plugin.py`:

```python
class LTVPlugin:
    def info(self):
        return {
            'id': 'my_plugin',
            'name': 'My Plugin',
            'name_cn': '我的插件',
            'description': 'A simple plugin',
            'description_cn': '一个简单的插件',
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
        return {'items': []}
    
    def process_line(self, line):
        return [{'text': line}]
```

3. **重启 Large Text Viewer**，插件会自动加载。

## Python 插件开发

### 安装和配置

#### 系统要求

- Python 3.8 或更高版本（需要支持共享库）
- Large Text Viewer 编译时启用 `python-plugins` 特性

#### 编译支持 Python 插件

```bash
# 设置 Python 路径（如果需要）
export PYO3_PYTHON=/usr/local/bin/python3

# 编译 release 版本
cargo build --release --features python-plugins

# 或编译不带 Python 插件的版本
cargo build --release
```

**注意**: macOS 系统自带的 Python 可能不支持共享库嵌入。建议使用 python.org 下载的官方 Python 或 Homebrew 安装的 Python。

#### 插件目录

插件文件放置在以下目录:

- **macOS/Linux**: `~/.large-text-viewer/plugins/`
- **Windows**: `%USERPROFILE%\.large-text-viewer\plugins\`

### 插件结构

每个 Python 插件是一个 `.py` 文件，必须包含一个名为 `LTVPlugin` 的类。

```python
#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""插件描述"""

from typing import Dict, List, Any

class LTVPlugin:
    """插件主类 - 必须命名为 LTVPlugin"""
    
    def __init__(self):
        """构造函数 - 初始化状态"""
        self.is_active = False
    
    def info(self) -> Dict[str, str]:
        """返回插件元信息"""
        pass
    
    def can_handle(self, content_sample: str) -> bool:
        """检测是否可以处理此文件"""
        pass
    
    def on_activate(self, content_sample: str):
        """插件激活时调用"""
        pass
    
    def on_deactivate(self):
        """插件停用时调用"""
        pass
    
    def get_panel_data(self) -> Dict[str, Any]:
        """返回侧边面板数据"""
        pass
    
    def process_line(self, line: str) -> List[Dict[str, Any]]:
        """处理单行，返回高亮片段"""
        pass
    
    def on_action(self, action: str):
        """处理按钮点击动作（可选）"""
        pass
```

### API 参考

#### `info() -> Dict[str, str]`

返回插件元信息字典。

| 键 | 类型 | 必须 | 说明 |
|----|------|------|------|
| `id` | str | ✓ | 唯一标识符（英文，无空格） |
| `name` | str | ✓ | 英文名称 |
| `name_cn` | str | ✓ | 中文名称 |
| `description` | str | ✓ | 英文描述 |
| `description_cn` | str | ✓ | 中文描述 |
| `version` | str | ✓ | 版本号（如 "1.0.0"） |
| `author` | str | ✓ | 作者名称 |
| `icon` | str | ✓ | Emoji 图标 |

```python
def info(self):
    return {
        'id': 'json_viewer',
        'name': 'JSON Viewer',
        'name_cn': 'JSON 查看器',
        'description': 'Syntax highlighting for JSON files',
        'description_cn': 'JSON 文件语法高亮',
        'version': '1.0.0',
        'author': 'Developer',
        'icon': '📋',
    }
```

#### `can_handle(content_sample: str) -> bool`

检测是否可以处理给定的文件内容。

**参数**:
- `content_sample`: 文件内容的前几 KB 样本

**返回**: 如果可以处理返回 `True`，否则返回 `False`

```python
def can_handle(self, content_sample):
    # 检查是否为 JSON 格式
    content = content_sample.strip()
    if content.startswith('{') or content.startswith('['):
        try:
            import json
            json.loads(content_sample[:10000])
            return True
        except:
            pass
    return False
```

#### `on_activate(content_sample: str)`

插件激活时调用，用于初始化和分析数据。

```python
def on_activate(self, content_sample):
    self.is_active = True
    # 分析内容
    self.line_count = content_sample.count('\n') + 1
```

#### `on_deactivate()`

插件停用时调用，用于清理资源。

```python
def on_deactivate(self):
    self.is_active = False
    self.line_count = 0
```

#### `get_panel_data() -> Dict[str, Any]`

返回侧边面板的 UI 数据。

**返回格式**:

```python
{
    'items': [
        # 面板项列表
    ]
}
```

**支持的面板项类型**:

##### 1. 分节标题 (section)

```python
{'type': 'section', 'label': '📊 统计信息'}
```

##### 2. 统计项 (stat)

```python
{
    'type': 'stat',
    'label': '总行数',
    'value': '1234',
    'color': [99, 179, 237]  # RGB，可选
}
```

##### 3. 分隔线 (separator)

```python
{'type': 'separator'}
```

##### 4. 按钮 (button)

```python
{
    'type': 'button',
    'label': '🔄 刷新',
    'action': 'refresh'  # 传递给 on_action 的标识
}
```

##### 5. 普通标签 (label)

```python
{
    'type': 'label',
    'label': '提示文字',
    'color': [150, 150, 150]  # RGB，可选
}
```

**完整示例**:

```python
def get_panel_data(self):
    return {
        'items': [
            {'type': 'section', 'label': '📊 统计'},
            {'type': 'stat', 'label': '行数', 'value': '100', 'color': [99, 179, 237]},
            {'type': 'stat', 'label': '字符', 'value': '5000', 'color': [152, 195, 121]},
            {'type': 'separator'},
            {'type': 'section', 'label': '⚙️ 操作'},
            {'type': 'button', 'label': '刷新分析', 'action': 'refresh'},
            {'type': 'separator'},
            {'type': 'label', 'label': '提示：点击刷新更新数据', 'color': [150, 150, 150]},
        ]
    }
```

#### `process_line(line: str) -> List[Dict[str, Any]]`

处理单行文本，返回高亮片段列表。

**参数**:
- `line`: 要处理的行文本

**返回**: 高亮片段列表

**片段格式**:

```python
{
    'text': '文本内容',      # 必须
    'color': [R, G, B],    # RGB，可选
    'bold': True/False,    # 可选
    'italic': True/False,  # 可选
}
```

**示例**:

```python
def process_line(self, line):
    segments = []
    
    # 注释行
    if line.strip().startswith('#'):
        return [{'text': line, 'color': [150, 150, 150], 'italic': True}]
    
    # 关键词高亮
    import re
    pos = 0
    for match in re.finditer(r'\b(ERROR|WARN|INFO)\b', line):
        # 匹配前的文本
        if match.start() > pos:
            segments.append({'text': line[pos:match.start()]})
        
        # 高亮关键词
        keyword = match.group()
        color = {
            'ERROR': [224, 108, 117],  # 红色
            'WARN': [229, 192, 123],   # 黄色
            'INFO': [97, 175, 239],    # 蓝色
        }.get(keyword, [171, 178, 191])
        
        segments.append({
            'text': keyword,
            'color': color,
            'bold': True
        })
        pos = match.end()
    
    # 剩余文本
    if pos < len(line):
        segments.append({'text': line[pos:]})
    
    return segments if segments else [{'text': line}]
```

#### `on_action(action: str)` (可选)

处理按钮点击等动作。

```python
def on_action(self, action):
    if action == 'refresh':
        self._refresh_data()
    elif action == 'toggle_filter':
        self.filter_enabled = not self.filter_enabled
```

### 示例插件

查看 `examples/plugins/` 目录获取完整示例:

- `json_viewer.py` - JSON 文件语法高亮
- `log_analyzer.py` - 日志文件分析
- `plugin_template.py` - 插件模板

## Rust 插件开发

### Plugin Trait

Rust 插件需要实现 `Plugin` trait:

```rust
pub trait Plugin: Send {
    /// 获取插件信息
    fn info(&self) -> &PluginInfo;
    
    /// 检测是否可以处理此文件内容
    fn can_handle(&self, content_sample: &str) -> bool;
    
    /// 插件激活时调用
    fn on_activate(&mut self, content_sample: &str);
    
    /// 插件停用时调用
    fn on_deactivate(&mut self);
    
    /// 是否需要侧边面板
    fn has_side_panel(&self) -> bool { true }
    
    /// 渲染侧边面板
    fn render_side_panel(&mut self, ui: &mut egui::Ui, ctx: &PluginContext);
    
    /// 渲染单行内容
    fn render_line(&mut self, ui: &mut egui::Ui, line: &str, ctx: &PluginContext);
    
    // 可选方法...
}
```

### 内置插件示例

参考 `src/trace_plugin.rs` 查看完整的 Rust 插件实现。

### 注册内置插件

在 `src/app.rs` 的 `Default` 实现中注册:

```rust
plugin_manager: {
    let mut pm = PluginManager::new();
    pm.register(Box::new(MyPlugin::new()));
    pm
},
```

## 最佳实践

### 性能优化

1. **缓存处理结果**:

```python
def __init__(self):
    self._line_cache = {}

def process_line(self, line):
    if line in self._line_cache:
        return self._line_cache[line]
    result = self._process(line)
    if len(self._line_cache) < 10000:  # 限制缓存大小
        self._line_cache[line] = result
    return result
```

2. **避免复杂的正则**:

```python
# 不好：每次调用都编译正则
def process_line(self, line):
    import re
    match = re.search(r'complex_pattern', line)

# 好：预编译正则
def __init__(self):
    import re
    self._pattern = re.compile(r'complex_pattern')

def process_line(self, line):
    match = self._pattern.search(line)
```

3. **限制分析范围**:

```python
def on_activate(self, content_sample):
    # 只分析前 N 行
    lines = content_sample.split('\n')[:1000]
    self._analyze(lines)
```

### 错误处理

```python
def can_handle(self, content_sample):
    try:
        return self._detect(content_sample)
    except Exception:
        return False

def process_line(self, line):
    try:
        return self._highlight(line)
    except Exception:
        return [{'text': line}]  # 降级为普通文本
```

### UI 设计

1. **合理使用颜色**:
   - 使用一致的颜色方案
   - 考虑深色/浅色主题
   - 不要使用太多颜色

2. **面板布局**:
   - 使用 section 组织内容
   - 最重要的信息放在顶部
   - 使用 separator 分隔不同区域

## 常见问题

### Q: 插件没有加载？

1. 检查文件是否在正确的目录
2. 检查文件名是否以 `.py` 结尾
3. 检查是否以 `_` 开头（会被忽略）
4. 查看控制台错误信息

### Q: 插件没有被自动激活？

检查 `can_handle()` 方法的返回值和逻辑。

### Q: 高亮不正确？

1. 检查 `process_line()` 返回的片段是否完整覆盖整行
2. 确保颜色值是 `[R, G, B]` 格式

### Q: 面板不显示？

1. 检查 `get_panel_data()` 返回格式
2. 确保返回 `{'items': [...]}` 结构

## 更新日志

- **1.0.0**: 初始版本，支持 Python 3 插件

