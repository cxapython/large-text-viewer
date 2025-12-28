#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Large Text Viewer 插件模板

这是一个插件模板，展示了所有可用的 API。
复制此文件并修改以创建你自己的插件。

使用方法:
1. 复制此文件到 ~/.large-text-viewer/plugins/ 目录
2. 重命名文件（如 my_plugin.py）
3. 修改 LTVPlugin 类实现你的功能
4. 重启 Large Text Viewer
"""

from typing import Dict, List, Any, Optional


class LTVPlugin:
    """
    Large Text Viewer 插件基类
    
    必须实现的方法:
    - info() -> dict: 返回插件元信息
    - can_handle(content_sample: str) -> bool: 检测是否可以处理此文件
    - on_activate(content_sample: str): 插件激活时调用
    - on_deactivate(): 插件停用时调用
    - get_panel_data() -> dict: 返回侧边面板数据
    - process_line(line: str) -> list: 处理单行，返回高亮片段
    
    可选方法:
    - on_action(action: str): 处理按钮点击等动作
    """
    
    def __init__(self):
        """
        插件构造函数
        
        在这里初始化插件状态变量。
        注意：这个方法在插件加载时调用，不是激活时。
        """
        self.is_active = False
        # 在这里添加你的状态变量
        self.my_data = {}
    
    def info(self) -> Dict[str, str]:
        """
        返回插件元信息
        
        Returns:
            包含以下键的字典：
            - id: 插件唯一标识符（英文，无空格）
            - name: 插件英文名称
            - name_cn: 插件中文名称
            - description: 插件英文描述
            - description_cn: 插件中文描述
            - version: 版本号（如 "1.0.0"）
            - author: 作者名称
            - icon: 插件图标（emoji）
        """
        return {
            'id': 'my_plugin',           # 唯一标识符
            'name': 'My Plugin',         # 英文名称
            'name_cn': '我的插件',        # 中文名称
            'description': 'A template plugin',  # 英文描述
            'description_cn': '一个模板插件',     # 中文描述
            'version': '1.0.0',          # 版本号
            'author': 'Your Name',       # 作者
            'icon': '🔧',                # emoji 图标
        }
    
    def can_handle(self, content_sample: str) -> bool:
        """
        检测是否可以处理此文件
        
        当用户打开文件时，会调用所有插件的此方法。
        第一个返回 True 的插件会被自动激活。
        
        Args:
            content_sample: 文件内容的前几 KB 样本
            
        Returns:
            如果此插件可以处理该文件类型，返回 True
            
        Tips:
            - 检查文件头部标识
            - 检查特定的模式或关键字
            - 不要进行复杂的解析，保持快速
        """
        # 示例：检查文件是否包含特定标记
        return 'MY_SPECIAL_MARKER' in content_sample
    
    def on_activate(self, content_sample: str):
        """
        插件激活时调用
        
        当插件被选中（自动或手动）时调用。
        在这里进行初始化和数据分析。
        
        Args:
            content_sample: 文件内容样本
            
        Tips:
            - 分析内容统计信息
            - 初始化过滤器状态
            - 不要存储大量数据
        """
        self.is_active = True
        # 在这里分析内容
        self._analyze(content_sample)
    
    def on_deactivate(self):
        """
        插件停用时调用
        
        清理资源和重置状态。
        """
        self.is_active = False
        self.my_data = {}
    
    def _analyze(self, content: str):
        """
        内部分析方法
        
        这是一个私有方法示例，用于分析内容。
        """
        # 实现你的分析逻辑
        lines = content.split('\n')
        self.my_data['line_count'] = len(lines)
    
    def get_panel_data(self) -> Dict[str, Any]:
        """
        返回侧边面板数据
        
        定义侧边面板的 UI 内容。每次面板刷新时调用。
        
        Returns:
            包含 'items' 键的字典，值为面板项列表。
            
        支持的面板项类型:
        
        1. 分节标题 (section):
           {'type': 'section', 'label': '标题文字'}
           
        2. 统计项 (stat):
           {
               'type': 'stat',
               'label': '标签',
               'value': '值',
               'color': [R, G, B]  # 可选，RGB 颜色
           }
           
        3. 分隔线 (separator):
           {'type': 'separator'}
           
        4. 按钮 (button):
           {
               'type': 'button',
               'label': '按钮文字',
               'action': 'action_id'  # 点击时传递给 on_action
           }
           
        5. 普通标签 (label):
           {
               'type': 'label',
               'label': '文字内容',
               'color': [R, G, B]  # 可选
           }
        """
        return {
            'items': [
                # 统计区域
                {'type': 'section', 'label': '📊 统计信息'},
                {
                    'type': 'stat',
                    'label': '行数',
                    'value': str(self.my_data.get('line_count', 0)),
                    'color': [99, 179, 237]  # 蓝色
                },
                
                {'type': 'separator'},
                
                # 操作按钮
                {'type': 'section', 'label': '⚙️ 操作'},
                {
                    'type': 'button',
                    'label': '🔄 刷新',
                    'action': 'refresh'
                },
                
                {'type': 'separator'},
                
                # 说明文字
                {
                    'type': 'label',
                    'label': '这是一个模板插件',
                    'color': [150, 150, 150]  # 灰色
                },
            ]
        }
    
    def process_line(self, line: str) -> List[Dict[str, Any]]:
        """
        处理单行并返回高亮片段
        
        为每一行内容生成语法高亮。
        
        Args:
            line: 要处理的行文本
            
        Returns:
            高亮片段列表。每个片段是一个字典：
            {
                'text': '文本内容',          # 必须
                'color': [R, G, B],         # 可选，RGB 颜色
                'bold': True/False,         # 可选，是否粗体
                'italic': True/False,       # 可选，是否斜体
            }
            
        Tips:
            - 片段按顺序拼接形成完整行
            - 如果不需要高亮，返回 [{'text': line}]
            - 性能很重要，避免复杂的正则
        """
        # 简单示例：高亮特定关键词
        segments = []
        
        # 示例颜色
        KEYWORD_COLOR = [224, 108, 117]   # 红色
        COMMENT_COLOR = [150, 150, 150]   # 灰色
        DEFAULT_COLOR = [171, 178, 191]   # 默认
        
        # 简单的关键词高亮示例
        if line.strip().startswith('#'):
            # 注释行
            segments.append({
                'text': line,
                'color': COMMENT_COLOR,
                'italic': True,
            })
        elif 'KEYWORD' in line:
            # 包含关键词的行
            parts = line.split('KEYWORD')
            for i, part in enumerate(parts):
                if part:
                    segments.append({
                        'text': part,
                        'color': DEFAULT_COLOR,
                    })
                if i < len(parts) - 1:
                    segments.append({
                        'text': 'KEYWORD',
                        'color': KEYWORD_COLOR,
                        'bold': True,
                    })
        else:
            # 普通行
            segments.append({
                'text': line,
                'color': DEFAULT_COLOR,
            })
        
        return segments
    
    def on_action(self, action: str):
        """
        处理按钮点击等动作
        
        当用户点击面板中的按钮时调用。
        
        Args:
            action: 按钮定义中的 'action' 值
        """
        if action == 'refresh':
            # 处理刷新按钮
            print('刷新按钮被点击')
        elif action == 'another_action':
            # 处理其他动作
            pass


# ========== 高级技巧 ==========

"""
1. 使用缓存提高性能:

   def __init__(self):
       self._cache = {}
   
   def process_line(self, line: str):
       if line in self._cache:
           return self._cache[line]
       result = self._expensive_process(line)
       self._cache[line] = result
       return result

2. 使用正则表达式进行复杂匹配:

   import re
   
   def process_line(self, line: str):
       pattern = r'(\d{4}-\d{2}-\d{2})'
       # ...

3. 状态过滤:

   def __init__(self):
       self.filter_enabled = True
   
   def get_panel_data(self):
       return {
           'items': [
               {
                   'type': 'button',
                   'label': '启用过滤' if not self.filter_enabled else '禁用过滤',
                   'action': 'toggle_filter'
               }
           ]
       }
   
   def on_action(self, action):
       if action == 'toggle_filter':
           self.filter_enabled = not self.filter_enabled

4. 错误处理:

   def can_handle(self, content_sample):
       try:
           # 可能失败的检测逻辑
           return self._detect(content_sample)
       except Exception:
           return False
"""

