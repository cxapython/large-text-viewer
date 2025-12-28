#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
日志分析器插件

支持常见日志格式的语法高亮和统计分析：
- ERROR / WARN / INFO / DEBUG 级别
- 时间戳高亮
- IP 地址识别
- HTTP 状态码
"""

import re
from typing import Dict, List, Any
from datetime import datetime


class LTVPlugin:
    """日志文件分析插件"""
    
    def __init__(self):
        self.is_active = False
        self.stats = {
            'total_lines': 0,
            'error_count': 0,
            'warn_count': 0,
            'info_count': 0,
            'debug_count': 0,
            'unique_ips': set(),
            'http_2xx': 0,
            'http_3xx': 0,
            'http_4xx': 0,
            'http_5xx': 0,
        }
        # 日志级别过滤
        self.show_levels = {
            'ERROR': True,
            'WARN': True,
            'INFO': True,
            'DEBUG': True,
        }
    
    def info(self) -> Dict[str, str]:
        return {
            'id': 'log_analyzer',
            'name': 'Log Analyzer',
            'name_cn': '日志分析器',
            'description': 'Log file syntax highlighting and analysis',
            'description_cn': '日志文件语法高亮和分析',
            'version': '1.0.0',
            'author': 'Example',
            'icon': '📜',
        }
    
    def can_handle(self, content_sample: str) -> bool:
        """检测是否为日志文件"""
        lines = content_sample.split('\n')[:20]
        
        # 日志特征模式
        log_patterns = [
            r'\b(ERROR|WARN|INFO|DEBUG|TRACE|FATAL)\b',  # 日志级别
            r'\d{4}[-/]\d{2}[-/]\d{2}',                  # 日期
            r'\d{2}:\d{2}:\d{2}',                        # 时间
            r'\[\w+\]',                                  # [TAG] 格式
            r'^\d+\.\d+\.\d+\.\d+',                     # IP 地址开头
        ]
        
        matches = 0
        for line in lines:
            for pattern in log_patterns:
                if re.search(pattern, line, re.IGNORECASE):
                    matches += 1
                    break
        
        # 如果超过 30% 的行匹配日志模式
        return matches > len(lines) * 0.3
    
    def on_activate(self, content_sample: str):
        self.is_active = True
        self._analyze_logs(content_sample)
    
    def on_deactivate(self):
        self.is_active = False
        self.stats = {
            'total_lines': 0,
            'error_count': 0,
            'warn_count': 0,
            'info_count': 0,
            'debug_count': 0,
            'unique_ips': set(),
            'http_2xx': 0,
            'http_3xx': 0,
            'http_4xx': 0,
            'http_5xx': 0,
        }
    
    def _analyze_logs(self, content: str):
        """分析日志内容"""
        lines = content.split('\n')
        self.stats['total_lines'] = len(lines)
        
        for line in lines:
            line_upper = line.upper()
            
            # 统计日志级别
            if 'ERROR' in line_upper or 'FATAL' in line_upper:
                self.stats['error_count'] += 1
            elif 'WARN' in line_upper:
                self.stats['warn_count'] += 1
            elif 'INFO' in line_upper:
                self.stats['info_count'] += 1
            elif 'DEBUG' in line_upper or 'TRACE' in line_upper:
                self.stats['debug_count'] += 1
            
            # 提取 IP 地址
            ips = re.findall(r'\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b', line)
            self.stats['unique_ips'].update(ips)
            
            # HTTP 状态码统计
            status_codes = re.findall(r'\b([2-5]\d{2})\b', line)
            for code in status_codes:
                code_int = int(code)
                if 200 <= code_int < 300:
                    self.stats['http_2xx'] += 1
                elif 300 <= code_int < 400:
                    self.stats['http_3xx'] += 1
                elif 400 <= code_int < 500:
                    self.stats['http_4xx'] += 1
                elif 500 <= code_int < 600:
                    self.stats['http_5xx'] += 1
    
    def get_panel_data(self) -> Dict[str, Any]:
        items = [
            {'type': 'section', 'label': '📊 日志级别统计'},
            {'type': 'stat', 'label': 'ERROR', 'value': str(self.stats['error_count']), 
             'color': [224, 108, 117]},  # 红色
            {'type': 'stat', 'label': 'WARN', 'value': str(self.stats['warn_count']), 
             'color': [229, 192, 123]},  # 黄色
            {'type': 'stat', 'label': 'INFO', 'value': str(self.stats['info_count']), 
             'color': [97, 175, 239]},   # 蓝色
            {'type': 'stat', 'label': 'DEBUG', 'value': str(self.stats['debug_count']), 
             'color': [150, 150, 150]},  # 灰色
            {'type': 'separator'},
            {'type': 'section', 'label': '🌐 网络统计'},
            {'type': 'stat', 'label': '唯一 IP', 'value': str(len(self.stats['unique_ips'])), 
             'color': [152, 195, 121]},
        ]
        
        # HTTP 状态码（如果有）
        total_http = self.stats['http_2xx'] + self.stats['http_3xx'] + \
                     self.stats['http_4xx'] + self.stats['http_5xx']
        if total_http > 0:
            items.extend([
                {'type': 'separator'},
                {'type': 'section', 'label': '📡 HTTP 状态'},
                {'type': 'stat', 'label': '2xx 成功', 'value': str(self.stats['http_2xx']), 
                 'color': [152, 195, 121]},
                {'type': 'stat', 'label': '3xx 重定向', 'value': str(self.stats['http_3xx']), 
                 'color': [99, 179, 237]},
                {'type': 'stat', 'label': '4xx 客户端错误', 'value': str(self.stats['http_4xx']), 
                 'color': [229, 192, 123]},
                {'type': 'stat', 'label': '5xx 服务器错误', 'value': str(self.stats['http_5xx']), 
                 'color': [224, 108, 117]},
            ])
        
        return {'items': items}
    
    def process_line(self, line: str) -> List[Dict[str, Any]]:
        """处理日志行高亮"""
        segments = []
        
        # 颜色定义
        COLORS = {
            'error': [224, 108, 117],    # 红色
            'warn': [229, 192, 123],     # 黄色
            'info': [97, 175, 239],      # 蓝色
            'debug': [150, 150, 150],    # 灰色
            'timestamp': [152, 195, 121], # 绿色
            'ip': [198, 120, 221],       # 紫色
            'http_ok': [152, 195, 121],  # 绿色
            'http_warn': [229, 192, 123], # 黄色
            'http_error': [224, 108, 117], # 红色
            'bracket': [99, 179, 237],   # 蓝色
            'default': [171, 178, 191],
        }
        
        pos = 0
        
        # 定义高亮模式
        patterns = [
            # 时间戳
            (r'\d{4}[-/]\d{2}[-/]\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?', 'timestamp'),
            (r'\d{2}:\d{2}:\d{2}(?:\.\d+)?', 'timestamp'),
            # 日志级别
            (r'\b(ERROR|FATAL)\b', 'error'),
            (r'\b(WARN(?:ING)?)\b', 'warn'),
            (r'\b(INFO)\b', 'info'),
            (r'\b(DEBUG|TRACE)\b', 'debug'),
            # IP 地址
            (r'\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b', 'ip'),
            # HTTP 状态码
            (r'\b(2\d{2})\b', 'http_ok'),
            (r'\b(3\d{2})\b', 'http_ok'),
            (r'\b(4\d{2})\b', 'http_warn'),
            (r'\b(5\d{2})\b', 'http_error'),
            # 方括号内容
            (r'\[[^\]]+\]', 'bracket'),
        ]
        
        # 找到所有匹配并按位置排序
        all_matches = []
        for pattern, color_name in patterns:
            for match in re.finditer(pattern, line, re.IGNORECASE):
                all_matches.append((match.start(), match.end(), match.group(), color_name))
        
        # 按起始位置排序
        all_matches.sort(key=lambda x: x[0])
        
        # 合并重叠的匹配
        filtered_matches = []
        last_end = 0
        for start, end, text, color_name in all_matches:
            if start >= last_end:
                filtered_matches.append((start, end, text, color_name))
                last_end = end
        
        # 生成高亮片段
        for start, end, text, color_name in filtered_matches:
            if start > pos:
                segments.append({
                    'text': line[pos:start],
                    'color': COLORS['default'],
                })
            
            color = COLORS.get(color_name, COLORS['default'])
            bold = color_name in ('error', 'warn')
            
            segments.append({
                'text': text,
                'color': color,
                'bold': bold,
            })
            pos = end
        
        # 添加剩余文本
        if pos < len(line):
            segments.append({
                'text': line[pos:],
                'color': COLORS['default'],
            })
        
        if not segments:
            segments.append({'text': line})
        
        return segments
    
    def on_action(self, action: str):
        """处理按钮动作"""
        if action.startswith('toggle_'):
            level = action[7:].upper()
            if level in self.show_levels:
                self.show_levels[level] = not self.show_levels[level]

