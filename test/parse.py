import re
from datetime import datetime, timedelta


def chinese_to_number(chinese_str):
    """将中文数字转换为阿拉伯数字"""
    chinese_num_map = {
        '零': 0, '一': 1, '二': 2, '两': 2, '三': 3, '四': 4, '五': 5,
        '六': 6, '七': 7, '八': 8, '九': 9, '十': 10,
        '十一': 11, '十二': 12, '二十': 20, '三十': 30
    }
    
    # 处理特殊情况
    if chinese_str in chinese_num_map:
        return chinese_num_map[chinese_str]
    
    # 处理十几、几十几的情况
    if '十' in chinese_str:
        if chinese_str.startswith('十'):
            return 10 + chinese_to_number(chinese_str[1:]) if len(chinese_str) > 1 else 10
        parts = chinese_str.split('十')
        return chinese_to_number(parts[0]) * 10 + (chinese_to_number(parts[1]) if parts[1] else 0)
    
    return 0  # 默认返回0


def parse_date(date_str):
    """解析日期字符串，返回年、月、日"""
    now = datetime.now()
    year, month, day = now.year, now.month, now.day
    
    # 处理"明天"、"后天"等
    if '明天' in date_str:
        return (year, month, (now + timedelta(days=1)).day)
    elif '后天' in date_str:
        return (year, month, (now + timedelta(days=2)).day)
    
    # 处理"x月x日/号"格式
    month_match = re.search(r'((\d+)|([一二三四五六七八九十]+))月', date_str)
    day_match = re.search(r'((\d+)|([一二三四五六七八九十]+))([日号])', date_str)
    
    if month_match:
        month_str = month_match.group(1)
        if month_str.isdigit():
            month = int(month_str)
        else:
            month = chinese_to_number(month_str)
    
    if day_match:
        day_str = day_match.group(1)
        if day_str.isdigit():
            day = int(day_str)
        else:
            day = chinese_to_number(day_str)
    
    # 处理跨年情况
    if month < now.month:
        year += 1
        
    return (year, month, day)


def parse_reminder(sentence):
    # 首先检查是否是陈述性语句而非提醒请求
    # 过滤掉包含过去时间词汇的句子
    past_time_indicators = ['昨天', '前天', '上周', '上个月', '刚才', '刚刚', '已经']
    narrative_indicators = ['，我', '，有人', '的时候']
    future_statements = ['等等', '等会', '一会儿', '待会']
    
    # 检查是否包含过去时间指示词
    if any(indicator in sentence for indicator in past_time_indicators):
        return None, None
        
    # 检查是否是叙述性语句（通常包含逗号后接主语）
    if any(indicator in sentence for indicator in narrative_indicators):
        return None, None
        
    # 检查是否是未来时间陈述但不是提醒请求
    if any(indicator in sentence for indicator in future_statements) and '提醒' in sentence and not re.search(r'提醒我|喊我|叫我|记得', sentence):
        return None, None
    
    # 时间段映射表
    time_period_map = {
        '凌晨': (0, 5),    # 0:00-5:59
        '早上': (6, 8),    # 6:00-8:59
        '上午': (9, 11),   # 9:00-11:59
        '中午': (12, 13),  # 12:00-13:59
        '下午': (14, 17),  # 14:00-17:59
        '晚上': (18, 23),  # 18:00-23:59
        '傍晚': (17, 19),  # 17:00-19:59
        '夜晚': (20, 23)   # 20:00-23:59
    }
    
    # 正则表达式匹配时间部分和事件
    pattern = re.compile(
        # 相对时间
        r'(?:(?P<relative_num>\d+|[一二三四五六七八九十]+)(?P<relative_unit>分钟|小时|天)后)|'
        # 绝对日期和时间
        r'(?:(?P<date>明天|后天|(?:(?:\d+|[一二三四五六七八九十]+)月)?(?:(?:\d+|[一二三四五六七八九十]+)[日号])?)'
        r'(?:(?P<day_period>早上|上午|中午|下午|晚上|凌晨|傍晚|夜晚)?'
        r'(?:(?P<abs_hour>\d+|[一二三四五六七八九十]+)(?:点|时|:)?'
        r'(?P<half>半)?'  # 添加半小时的匹配
        r'(?P<abs_minute>\d+|[一二三四五六七八九十]+)?分?)?)?)'
        # 事件
        r'(?:.*?)(?:提醒我|喊我|叫我|记得|)(?P<event>.+)$',
        re.IGNORECASE
    )

    match = pattern.search(sentence)
    if not match:
        return None, None

    now = datetime.now()
    reminder_time = None
    event = match.group('event').strip() if match.group('event') else "未指定事件"

    # 处理相对时间
    if match.group('relative_num'):
        num_str = match.group('relative_num')
        num = int(num_str) if num_str.isdigit() else chinese_to_number(num_str)
        unit = match.group('relative_unit')
        
        if unit == '分钟':
            delta = timedelta(minutes=num)
        elif unit == '小时':
            delta = timedelta(hours=num)
        else:  # 天
            delta = timedelta(days=num)
            
        reminder_time = now + delta
    else:
        # 处理绝对时间
        date_str = match.group('date') or '明天'
        year, month, day = parse_date(date_str)
        
        # 默认时间8点
        hour = 8
        minute = 0

        # 处理小时
        if match.group('abs_hour'):
            hour_str = match.group('abs_hour')
            hour = int(hour_str) if hour_str.isdigit() else chinese_to_number(hour_str)
            
            # 处理时间段
            day_period = match.group('day_period')
            if day_period:
                period_range = time_period_map.get(day_period)
                if period_range:
                    # 如果小时数小于时间段的开始，且不是12点，则调整到对应时间段
                    if hour < period_range[0] and hour != 12:
                        hour += 12
                    # 特殊处理12点
                    elif hour == 12 and day_period in ['早上', '上午', '凌晨']:
                        hour = 0
        # 没有明确时间段但是下午的时间
        elif 1 <= hour <= 11 and now.hour >= 12:
            # 当前是下午且说的是小时数，默认也是下午
            hour += 12

        # 处理分钟
        if match.group('abs_minute'):
            minute_str = match.group('abs_minute')
            minute = int(minute_str) if minute_str.isdigit() else chinese_to_number(minute_str)
        # 处理半小时
        elif match.group('half'):
            minute = 30

        reminder_time = datetime(year=year, month=month, day=day, hour=hour, minute=minute)
        
        # 如果设置的时间已经过去，则加一天
        if reminder_time < now:
            reminder_time += timedelta(days=1)

    return reminder_time, event


# 使用示例
sentences = [
    "提醒我2分钟后睡觉",
    "三小时后提醒我关火",
    "明天早上七点喊我起床",
    "明天提醒我买菜",
    "明天下午3点30分开会",
    "三月二十八号提醒我买东西",
    "明天八点提醒我起床",
    "后天中午十二点提醒我吃饭",
    "昨天中午十二点，我妈妈提醒我吃饭了",
    "昨天早上8点去学校的时候，我妈提醒我带了饭盒，你可以今天也提醒我带饭盒吗",
    "下午五点半提醒我健身"
]

for text in sentences:
    time, event = parse_reminder(text)
    print(f"原句：{text}")
    print(f"解析结果：时间={time.strftime('%Y-%m-%d %H:%M') if time else None}, 事件='{event}'\n")
