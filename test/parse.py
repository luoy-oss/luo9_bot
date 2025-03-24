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
    # 过滤掉包含过去时间词汇的句子，但允许后面跟着新的提醒请求
    past_time_indicators = ['昨天', '前天', '上周', '上个月', '刚才', '刚刚', '已经']
    narrative_indicators = ['，我', '，有人', '的时候']
    future_statements = ['等等', '等会', '一会儿', '待会']
    
    # 新增：过滤掉非标准格式的提醒请求
    # 检查句子是否以提醒相关词语开头，或者包含明确的提醒格式
    standard_reminder_patterns = [
        r'^提醒我',
        r'^喊我',
        r'^叫我',
        r'^记得',
        r'^明天',
        r'^后天',
        r'^今天',
        r'^\d+月',
        r'^[一二三四五六七八九十]+月'
    ]
    
    # 如果句子不是以标准提醒格式开头，且包含感叹词或情绪表达，则认为不是有效的提醒请求
    emotion_indicators = ['啊', '哦', '呀', '哎', '唉', '嗯', '好困', '好累', '好烦', '好难']
    
    # 检查是否是非标准格式
    is_non_standard = True
    for pattern in standard_reminder_patterns:
        if re.search(pattern, sentence.strip()):
            is_non_standard = False
            break
    
    # 如果是非标准格式且包含情绪词，则直接返回None
    if is_non_standard and any(indicator in sentence for indicator in emotion_indicators):
        return None, None
    
    # 检查是否包含"今天也提醒我"或类似的表达，这表示是一个有效的提醒请求
    valid_request_patterns = [
        r'今天也(?:提醒|喊|叫)我',
        r'你(?:也)?可以(?:提醒|喊|叫)我',
        r'(?:请|麻烦)(?:提醒|喊|叫)我'
    ]
    
    # 如果句子中包含有效的提醒请求模式，则尝试从这部分提取信息
    for pattern in valid_request_patterns:
        if re.search(pattern, sentence):
            # 找到有效请求的起始位置
            match_obj = re.search(pattern, sentence)
            match_pos = match_obj.start()
            match_end = match_obj.end()
            
            # 提取事件部分（在"提醒我"之后的内容）
            event_part = sentence[match_end:]
            # 如果事件部分为空或只有标点符号，尝试从前面的句子中提取事件
            if not event_part.strip() or event_part.strip() in ['吗', '吗？', '？', '。']:
                # 尝试从前面的句子中找到可能的事件
                prev_event_match = re.search(r'提醒我(带了|带|拿了|拿|买了|买|做了|做)([^，。？！]+)', sentence[:match_pos])
                if prev_event_match:
                    # 使用前面句子中提到的事件，但去掉"了"字
                    action = prev_event_match.group(1).replace('了', '')
                    event = action + prev_event_match.group(2).strip()
                    # 构造新的句子用于解析时间
                    new_sentence = sentence[match_pos:] + f" {event}"
                    return parse_reminder_core(new_sentence, forced_event=event)
            
            # 只处理句子的后半部分
            return parse_reminder_core(sentence[match_pos:])
    
    # 如果没有找到有效的提醒请求模式，则检查是否应该过滤掉整个句子
    if any(indicator in sentence for indicator in past_time_indicators) and not re.search(r'今天|明天|后天', sentence):
        return None, None
        
    # 检查是否是叙述性语句（通常包含逗号后接主语）
    if any(indicator in sentence for indicator in narrative_indicators) and not re.search(r'今天|明天|后天', sentence):
        return None, None
        
    # 检查是否是未来时间陈述但不是提醒请求
    if any(indicator in sentence for indicator in future_statements) and '提醒' in sentence and not re.search(r'提醒我|喊我|叫我|记得', sentence):
        return None, None
    
    # 处理正常的提醒请求
    return parse_reminder_core(sentence)


def parse_reminder_core(sentence, forced_event=None):
    """核心提醒解析函数，处理已经过滤的句子"""
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
        r'(?:(?P<relative_num>\d+|[一二三四五六七八九十]+)(?P<relative_unit>分钟|小时|天)后)'
        r'(?:.*?)(?:提醒我|喊我|叫我|记得)(?P<relative_event>.+?)(?:\?|？|吗|吧|呢|啊|$)|'
        # 绝对日期和时间 + 事件（带提醒词）
        r'(?:记得)?(?:(?P<date>今天|明天|后天|(?:(?:\d+|[一二三四五六七八九十]+)月)?(?:(?:\d+|[一二三四五六七八九十]+)[日号])?)'
        r'(?:(?P<day_period>早上|上午|中午|下午|晚上|凌晨|傍晚|夜晚)?'
        r'(?:(?P<abs_hour>\d+|[一二三四五六七八九十]+)(?:点|时|:)?'
        r'(?P<half>半)?'  # 添加半小时的匹配
        r'(?P<abs_minute>\d+|[一二三四五六七八九十]+)?分?)?)?)'
        r'(?:.*?)(?:提醒我|喊我|叫我)(?P<abs_event>.+?)(?:\?|？|吗|吧|呢|啊|$)|'
        # 绝对日期和时间 + 事件（不带提醒词，直接是事件）
        r'(?:记得)?(?:(?P<date2>今天|明天|后天|(?:(?:\d+|[一二三四五六七八九十]+)月)?(?:(?:\d+|[一二三四五六七八九十]+)[日号])?)'
        r'(?:(?P<day_period2>早上|上午|中午|下午|晚上|凌晨|傍晚|夜晚)?'
        r'(?:(?P<abs_hour2>\d+|[一二三四五六七八九十]+)(?:点|时|:)?'
        r'(?P<half2>半)?'  # 添加半小时的匹配
        r'(?P<abs_minute2>\d+|[一二三四五六七八九十]+)?分?)?)?)'
        r'(?P<direct_event>[^，。？！]+)',
        re.IGNORECASE
    )

    match = pattern.search(sentence)
    if not match:
        return None, None

    now = datetime.now()
    reminder_time = None
    
    # 如果有强制指定的事件，则使用它
    if forced_event:
        event = forced_event
    else:
        # 根据匹配的模式选择对应的事件组
        if match.group('relative_num'):
            event = match.group('relative_event')
        elif match.group('abs_event'):
            event = match.group('abs_event')
        elif match.group('direct_event'):
            event = match.group('direct_event')
        else:
            event = "未指定事件"
            
        # 确保事件不为None
        event = event.strip() if event else "未指定事件"
        # 清理事件文本中可能的干扰词
        event = re.sub(r'^(也|帮我|帮忙|请|麻烦|可以|能不能|能否|能够)', '', event).strip()
        # 移除结尾的语气助词
        event = re.sub(r'(吧|呢|啊|哦|哈|呀)$', '', event).strip()
        # 如果事件为空，设为未指定
        if not event:
            event = "未指定事件"

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
        # 选择匹配到的日期组（可能是date或date2）
        date_str = match.group('date') or match.group('date2') or '今天'
        
        # 处理日期
        if date_str == '今天':
            year, month, day = now.year, now.month, now.day
        else:
            year, month, day = parse_date(date_str)
        
        # 默认时间8点
        hour = 8
        minute = 0

        # 处理小时（可能是abs_hour或abs_hour2）
        hour_str = match.group('abs_hour') or match.group('abs_hour2')
        if hour_str:
            hour = int(hour_str) if hour_str.isdigit() else chinese_to_number(hour_str)
            
            # 处理时间段（可能是day_period或day_period2）
            day_period = match.group('day_period') or match.group('day_period2')
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

        # 处理分钟（可能是abs_minute或abs_minute2）
        minute_str = match.group('abs_minute') or match.group('abs_minute2')
        if minute_str:
            minute = int(minute_str) if minute_str.isdigit() else chinese_to_number(minute_str)
        # 处理半小时（可能是half或half2）
        elif match.group('half') or match.group('half2'):
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
    "记得三月二十八号提醒我买东西",
    "明天八点提醒我起床",
    "后天中午十二点提醒我吃饭",
    "昨天中午十二点，我妈妈提醒我吃饭了",
    "昨天早上8点去学校的时候，我妈提醒我带了饭盒，你可以今天也提醒我带饭盒吗",
    "早上8点去学校的时候，提醒我带饭盒",
    "下午五点半记得提醒我健身",
    "2分钟后喊我睡觉吧"
]

for text in sentences:
    time, event = parse_reminder(text)
    print(f"原句：{text}")
    print(f"解析结果：时间={time.strftime('%Y-%m-%d %H:%M') if time else None}, 事件='{event}'\n")
