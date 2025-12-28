#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
JSON 查看器插件示例

这是一个简单的 JSON 文件语法高亮插件，展示如何编写 Large Text Viewer 的 Python 插件。
"""

import re
import json
from typing import Dict, List, Any, Optional


class LTVPlugin:
    """
    Large Text Viewer 插件基类
    
    所有插件都需要实现这个类，并提供以下方法：
    - info() -> dict
    - can_handle(content_sample: str) -> bool
    - on_activate(content_sample: str)
    - on_deactivate()
    - get_panel_data() -> dict
    - process_line(line: str) -> list
    """
    
    def __init__(self):
        self.is_active = False
        self.stats = {
            'total_lines': 0,
            'objects': 0,
            'arrays': 0,
            'strings': 0,
            'numbers': 0,
            'booleans': 0,
            'nulls': 0,
        }
        self.indent_level = 0
    
    def info(self) -> Dict[str, str]:
        """返回插件元信息"""
        return {
            'id': 'json_viewer',
            'name': 'JSON Viewer',
            'name_cn': 'JSON 查看器',
            'description': 'Syntax highlighting for JSON files',
            'description_cn': 'JSON 文件语法高亮',
            'version': '1.0.0',
            'author': 'Example',
            'icon': '📋',
        }
    
    def can_handle(self, content_sample: str) -> bool:
        """
        检测是否可以处理此文件
        
        Args:
            content_sample: 文件内容的前几行样本
            
        Returns:
            如果可以处理返回 True，否则返回 False
        """
        content = content_sample.strip()
        # 检查是否以 { 或 [ 开头（JSON 格式）
        if content.startswith('{') or content.startswith('['):
            try:
                # 尝试解析 JSON
                json.loads(content_sample[:10000])  # 只解析前 10000 字符
                return True
            except:
                # 可能是格式化的 JSON，逐行检查
                lines = content_sample.split('\n')[:10]
                json_patterns = [r'^\s*[\{\[\]\}]', r'^\s*"[^"]+"\s*:', r'^\s*\d+\s*,?\s*$']
                matches = sum(1 for line in lines for p in json_patterns if re.match(p, line))
                return matches >= 3
        return False
    
    def on_activate(self, content_sample: str):
        """
        插件激活时调用
        
        Args:
            content_sample: 文件内容样本，用于分析和初始化
        """
        self.is_active = True
        self._analyze_content(content_sample)
    
    def on_deactivate(self):
        """插件停用时调用"""
        self.is_active = False
        self.stats = {k: 0 for k in self.stats}
    
    def _analyze_content(self, content: str):
        """分析 JSON 内容统计信息"""
        self.stats['total_lines'] = content.count('\n') + 1
        self.stats['objects'] = content.count('{')
        self.stats['arrays'] = content.count('[')
        self.stats['strings'] = len(re.findall(r'"[^"]*"(?=\s*:)', content))
        self.stats['numbers'] = len(re.findall(r':\s*-?\d+\.?\d*', content))
        self.stats['booleans'] = content.count('true') + content.count('false')
        self.stats['nulls'] = content.count('null')
    
    def get_panel_data(self) -> Dict[str, Any]:
        """
        返回侧边面板数据
        
        Returns:
            包含面板项的字典，格式：
            {
                'items': [
                    {'type': 'section', 'label': '标题'},
                    {'type': 'stat', 'label': '标签', 'value': '值', 'color': [R, G, B]},
                    {'type': 'separator'},
                    {'type': 'button', 'label': '按钮', 'action': 'action_id'},
                    {'type': 'label', 'label': '普通文本', 'color': [R, G, B]},
                ]
            }
        """
        return {
            'items': [
                {'type': 'section', 'label': '📊 统计信息'},
                {'type': 'stat', 'label': '总行数', 'value': str(self.stats['total_lines']), 'color': [99, 179, 237]},
                {'type': 'stat', 'label': '对象 {}', 'value': str(self.stats['objects']), 'color': [152, 195, 121]},
                {'type': 'stat', 'label': '数组 []', 'value': str(self.stats['arrays']), 'color': [229, 192, 123]},
                {'type': 'separator'},
                {'type': 'section', 'label': '📝 数据类型'},
                {'type': 'stat', 'label': '键名', 'value': str(self.stats['strings']), 'color': [224, 108, 117]},
                {'type': 'stat', 'label': '数字', 'value': str(self.stats['numbers']), 'color': [209, 154, 102]},
                {'type': 'stat', 'label': '布尔', 'value': str(self.stats['booleans']), 'color': [198, 120, 221]},
                {'type': 'stat', 'label': 'null', 'value': str(self.stats['nulls']), 'color': [150, 150, 150]},
            ]
        }
    
    def process_line(self, line: str) -> List[Dict[str, Any]]:
        """
        处理单行并返回高亮片段
        
        Args:
            line: 要处理的行文本
            
        Returns:
            高亮片段列表，每个片段包含：
            {
                'text': '文本内容',
                'color': [R, G, B],  # 可选
                'bold': False,       # 可选
                'italic': False,     # 可选
            }
        """
        segments = []
        pos = 0
        
        # JSON 颜色定义
        COLORS = {
            'key': [224, 108, 117],      # 红色 - 键名
            'string': [152, 195, 121],   # 绿色 - 字符串值
            'number': [209, 154, 102],   # 橙色 - 数字
            'boolean': [198, 120, 221],  # 紫色 - 布尔
            'null': [150, 150, 150],     # 灰色 - null
            'bracket': [99, 179, 237],   # 蓝色 - 括号
            'colon': [171, 178, 191],    # 默认 - 冒号
            'comma': [171, 178, 191],    # 默认 - 逗号
        }
        
        # 正则模式
        patterns = [
            (r'"([^"\\]|\\.)*"\s*:', 'key'),        # 键名
            (r'"([^"\\]|\\.)*"', 'string'),         # 字符串值
            (r'-?\d+\.?\d*([eE][+-]?\d+)?', 'number'),  # 数字
            (r'\b(true|false)\b', 'boolean'),      # 布尔
            (r'\bnull\b', 'null'),                 # null
            (r'[\{\}\[\]]', 'bracket'),            # 括号
            (r':', 'colon'),                       # 冒号
            (r',', 'comma'),                       # 逗号
        ]
        
        # 合并所有模式
        combined_pattern = '|'.join(f'(?P<t{i}>{p[0]})' for i, p in enumerate(patterns))
        
        for match in re.finditer(combined_pattern, line):
            # 添加匹配前的普通文本
            if match.start() > pos:
                segments.append({
                    'text': line[pos:match.start()],
                })
            
            # 确定匹配类型
            matched_text = match.group()
            for i, (pattern, color_name) in enumerate(patterns):
                if match.group(f't{i}') is not None:
                    color = COLORS.get(color_name, [171, 178, 191])
                    # 键名特殊处理：去掉末尾的冒号
                    if color_name == 'key' and matched_text.endswith(':'):
                        segments.append({
                            'text': matched_text[:-1],
                            'color': color,
                            'bold': True,
                        })
                        segments.append({
                            'text': ':',
                            'color': COLORS['colon'],
                        })
                    else:
                        segments.append({
                            'text': matched_text,
                            'color': color,
                        })
                    break
            
            pos = match.end()
        
        # 添加剩余文本
        if pos < len(line):
            segments.append({
                'text': line[pos:],
            })
        
        # 如果没有任何匹配，返回原始行
        if not segments:
            segments.append({'text': line})
        
        return segments
    
    def on_action(self, action: str):
        """
        处理按钮点击等动作
        
        Args:
            action: 动作标识符
        """
        pass  # JSON 查看器暂无需要处理的动作

