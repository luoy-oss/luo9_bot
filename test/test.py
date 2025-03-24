import re
import datetime
import calendar
from enum import Enum

class RangeTimeEnum(Enum):
    """范围时间的默认时间点"""
    DAY_BREAK = 3      # 凌晨
    EARLY_MORNING = 8  # 早上
    MORNING = 10       # 上午
    NOON = 12          # 中午、午间
    AFTERNOON = 15     # 下午、午后
    NIGHT = 18         # 晚上、傍晚
    LATE_NIGHT = 20    # 晚、晚间
    MID_NIGHT = 23     # 深夜

class TimePoint:
    """时间表达式单元规范化对应的内部类"""
    def __init__(self):
        self.tunit = [-1, -1, -1, -1, -1, -1]  # 年-月-日-时-分-秒

class TimeUnit:
    """时间语句分析"""
    def __init__(self, time_expression, base_time=None):
        self.time_expression = time_expression
        self.time_norm = ""
        self.is_all_day_time = True
        self.is_first_time_solve_context = True
        
        self.tp = TimePoint()
        self.tp_origin = TimePoint()
        
        # 如果提供了基准时间，则使用基准时间
        if base_time:
            self.base_time = base_time
        else:
            self.base_time = datetime.datetime.now()
            
        self.time = None
        self.time_normalization()
    
    def time_normalization(self):
        """时间表达式规范化的主方法"""
        self.norm_set_year()
        self.norm_set_month()
        self.norm_set_day()
        self.norm_set_hour()
        self.norm_set_minute()
        self.norm_set_second()
        self.norm_set_total()
        self.norm_set_cur_related()
        self.norm_set_base_related()
        
        # 构建time对象
        time_grid = [self.tp.tunit[i] for i in range(0, 6)]
        
        # 检查是否需要设置默认值
        if time_grid[0] == -1:
            time_grid[0] = self.base_time.year
        if time_grid[1] == -1:
            time_grid[1] = self.base_time.month
        if time_grid[2] == -1:
            time_grid[2] = self.base_time.day
            
        # 将-1的部分设置为0
        for i in range(3, 6):
            if time_grid[i] == -1:
                time_grid[i] = 0
                
        try:
            self.time = datetime.datetime(
                time_grid[0], time_grid[1], time_grid[2], 
                time_grid[3], time_grid[4], time_grid[5]
            )
            self.prefer_future(3)  # 处理倾向于未来时间的情况
        except ValueError:
            # 处理无效日期，如2月30日
            self.time = self.base_time
    
    def norm_set_year(self):
        """年-规范化方法"""
        # 两位数年份
        rule = r"[0-9]{2}(?=年)"
        match = re.search(rule, self.time_expression)
        if match:
            year = int(match.group())
            if 0 <= year < 100:
                if year < 30:  # 30以下表示2000年以后的年份
                    year += 2000
                else:  # 否则表示1900年以后的年份
                    year += 1900
            self.tp.tunit[0] = year
            
        # 三位数和四位数年份
        rule = r"[0-9]?[0-9]{3}(?=年)"
        match = re.search(rule, self.time_expression)
        if match:  # 如果有3位数和4位数的年份，则覆盖原来2位数识别出的年份
            self.tp.tunit[0] = int(match.group())
    
    def norm_set_month(self):
        """月-规范化方法"""
        rule = r"((10)|(11)|(12)|([1-9]))(?=月)"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[1] = int(match.group())
            self.prefer_future(1)  # 处理倾向于未来时间的情况
            
        # 月-日 兼容模糊写法
        rule = r"((10)|(11)|(12)|([1-9]))(月|\.|\-)([0-3][0-9]|[1-9])"
        match = re.search(rule, self.time_expression)
        if match:
            match_str = match.group()
            p = re.compile(r"(月|\.|\-)")
            m = p.search(match_str)
            if m:
                split_index = m.start()
                month = match_str[:split_index]
                day = match_str[split_index + 1:]
                
                self.tp.tunit[1] = int(month)
                self.tp.tunit[2] = int(day)
                
                self.prefer_future(1)  # 处理倾向于未来时间的情况
    
    def norm_set_day(self):
        """日-规范化方法"""
        # 修改有问题的正则表达式，避免使用变长后向查找
        rule = r"([0-3][0-9]|[1-9])(?=(日|号))"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[2] = int(match.group(1))
            self.prefer_future(2)  # 处理倾向于未来时间的情况
    
    def norm_set_hour(self):
        """时-规范化方法"""
        # 修改有问题的正则表达式，避免使用变长后向查找
        rule = r"(?<![周星期])([0-2]?[0-9])(?=(点|时))"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[3] = int(match.group(1))
            self.prefer_future(3)  # 处理倾向于未来时间的情况
            self.is_all_day_time = False
            
        # 对关键字的处理：早上/早晨/早间，上午，中午,午间,下午,午后,晚上,傍晚,晚间,晚,pm,PM
        rule = r"凌晨"
        match = re.search(rule, self.time_expression)
        if match:
            if self.tp.tunit[3] == -1:  # 增加对没有明确时间点，只写了"凌晨"这种情况的处理
                self.tp.tunit[3] = RangeTimeEnum.DAY_BREAK.value
            self.prefer_future(3)
            self.is_all_day_time = False
            
        rule = r"早上|早晨|早间|晨间|今早|明早"
        match = re.search(rule, self.time_expression)
        if match:
            if self.tp.tunit[3] == -1:  # 增加对没有明确时间点，只写了"早上/早晨/早间"这种情况的处理
                self.tp.tunit[3] = RangeTimeEnum.EARLY_MORNING.value
            self.prefer_future(3)
            self.is_all_day_time = False
            
        rule = r"上午"
        match = re.search(rule, self.time_expression)
        if match:
            if self.tp.tunit[3] == -1:  # 增加对没有明确时间点，只写了"上午"这种情况的处理
                self.tp.tunit[3] = RangeTimeEnum.MORNING.value
            self.prefer_future(3)
            self.is_all_day_time = False
            
        rule = r"(中午)|(午间)"
        match = re.search(rule, self.time_expression)
        if match:
            if 0 <= self.tp.tunit[3] <= 10:
                self.tp.tunit[3] += 12
            if self.tp.tunit[3] == -1:  # 增加对没有明确时间点，只写了"中午/午间"这种情况的处理
                self.tp.tunit[3] = RangeTimeEnum.NOON.value
            self.prefer_future(3)
            self.is_all_day_time = False
            
        rule = r"(下午)|(午后)|(pm)|(PM)"
        match = re.search(rule, self.time_expression)
        if match:
            if 0 <= self.tp.tunit[3] <= 11:
                self.tp.tunit[3] += 12
            if self.tp.tunit[3] == -1:  # 增加对没有明确时间点，只写了"下午|午后"这种情况的处理
                self.tp.tunit[3] = RangeTimeEnum.AFTERNOON.value
            self.prefer_future(3)
            self.is_all_day_time = False
            
        rule = r"晚上|夜间|夜里|今晚|明晚"
        match = re.search(rule, self.time_expression)
        if match:
            if 1 <= self.tp.tunit[3] <= 11:
                self.tp.tunit[3] += 12
            elif self.tp.tunit[3] == 12:
                self.tp.tunit[3] = 0
            elif self.tp.tunit[3] == -1:
                self.tp.tunit[3] = RangeTimeEnum.NIGHT.value
            self.prefer_future(3)
            self.is_all_day_time = False
    
    def norm_set_minute(self):
        """分-规范化方法"""
        # 修改有问题的正则表达式，避免使用变长后向查找
        rule = r"([0-5]?[0-9](?=分(?!钟)))|([0-5]?[0-9](?!刻)(?<=\d[点时]))"
        match = re.search(rule, self.time_expression)
        if match and match.group():
            self.tp.tunit[4] = int(match.group())
            self.prefer_future(4)
            self.is_all_day_time = False
            
        # 加对一刻，半，3刻的正确识别（1刻为15分，半为30分，3刻为45分）
        rule = r"(?<=[点时])[1一]刻(?!钟)"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[4] = 15
            self.prefer_future(4)
            self.is_all_day_time = False
            
        rule = r"(?<=[点时])半"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[4] = 30
            self.prefer_future(4)
            self.is_all_day_time = False
            
        rule = r"(?<=[点时])[3三]刻(?!钟)"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[4] = 45
            self.prefer_future(4)
            self.is_all_day_time = False
    
    def norm_set_second(self):
        """秒-规范化方法"""
        rule = r"([0-5]?[0-9](?=秒))|((?<=分)[0-5]?[0-9])"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[5] = int(match.group())
            self.is_all_day_time = False
    
    def norm_set_total(self):
        """特殊形式的规范化方法"""
        # 修改有问题的正则表达式，避免使用变长后向查找
        rule = r"(?<![周星期])([0-2]?[0-9]):([0-5]?[0-9]):([0-5]?[0-9])"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[3] = int(match.group(1))
            self.tp.tunit[4] = int(match.group(2))
            self.tp.tunit[5] = int(match.group(3))
            self.prefer_future(3)
            self.is_all_day_time = False
        else:
            rule = r"(?<![周星期])([0-2]?[0-9]):([0-5]?[0-9])"
            match = re.search(rule, self.time_expression)
            if match:
                self.tp.tunit[3] = int(match.group(1))
                self.tp.tunit[4] = int(match.group(2))
                self.prefer_future(3)
                self.is_all_day_time = False
        
        # 增加了:固定形式时间表达式的正确识别
        rule = r"[0-9]?[0-9]?[0-9]{2}-((10)|(11)|(12)|([1-9]))-([0-3][0-9]|[1-9])"
        match = re.search(rule, self.time_expression)
        if match:
            tmp_target = match.group()
            tmp_parser = tmp_target.split("-")
            self.tp.tunit[0] = int(tmp_parser[0])
            self.tp.tunit[1] = int(tmp_parser[1])
            self.tp.tunit[2] = int(tmp_parser[2])
            
        rule = r"((10)|(11)|(12)|([1-9]))/([0-3][0-9]|[1-9])/[0-9]?[0-9]?[0-9]{2}"
        match = re.search(rule, self.time_expression)
        if match:
            tmp_target = match.group()
            tmp_parser = tmp_target.split("/")
            self.tp.tunit[1] = int(tmp_parser[0])
            self.tp.tunit[2] = int(tmp_parser[1])
            self.tp.tunit[0] = int(tmp_parser[2])
            
        rule = r"[0-9]?[0-9]?[0-9]{2}\.((10)|(11)|(12)|([1-9]))\.([0-3][0-9]|[1-9])"
        match = re.search(rule, self.time_expression)
        if match:
            tmp_target = match.group()
            tmp_parser = tmp_target.split(".")
            self.tp.tunit[0] = int(tmp_parser[0])
            self.tp.tunit[1] = int(tmp_parser[1])
            self.tp.tunit[2] = int(tmp_parser[2])
    
    def norm_set_cur_related(self):
        """设置当前时间相关的时间表达式"""
        rule = r"今天|今日|这天|这日|当日|当天"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month
            self.tp.tunit[2] = self.base_time.day
            
        rule = r"明天|明日"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month
            self.tp.tunit[2] = self.base_time.day + 1
            
        rule = r"后天|后日"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month
            self.tp.tunit[2] = self.base_time.day + 2
            
        rule = r"昨天|昨日|前天|前日"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month
            if "昨" in match.group():
                self.tp.tunit[2] = self.base_time.day - 1
            elif "前" in match.group():
                self.tp.tunit[2] = self.base_time.day - 2
                
        rule = r"上个月|上月|去几个月|去月"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month - 1
            if self.tp.tunit[1] == 0:
                self.tp.tunit[0] -= 1
                self.tp.tunit[1] = 12
                
        rule = r"下个月|下月|下一个月|下一月"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month + 1
            if self.tp.tunit[1] == 13:
                self.tp.tunit[0] += 1
                self.tp.tunit[1] = 1
                
        rule = r"大前天|大前日"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month
            self.tp.tunit[2] = self.base_time.day - 3
            
        rule = r"大后天|大后日"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            self.tp.tunit[1] = self.base_time.month
            self.tp.tunit[2] = self.base_time.day + 3
            
        rule = r"(?<=(上|去|前))[0-9]?[0-9](?=年)"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year - int(match.group())
            
        rule = r"(?<=(下|后))[0-9]?[0-9](?=年)"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year + int(match.group())
            
        rule = r"今年"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year
            
        rule = r"明年"
        match = re.search(rule, self.time_expression)
        if match:
            self.tp.tunit[0] = self.base_time.year + 1
            
        rule = r"去年|前年"
        match = re.search(rule, self.time_expression)
        if match:
            if "去" in match.group():
                self.tp.tunit[0] = self.base_time.year - 1
            elif "前" in match.group():
                self.tp.tunit[0] = self.base_time.year - 2
    
    def norm_set_base_related(self):
        """设置以某个时间为基准的时间表达式"""
        # 修改有问题的正则表达式，避免使用变长后向查找
        rule = r"(?<=[周星期])[1-7]"
        match = re.search(rule, self.time_expression)
        if match:
            week_day = int(match.group())
            if week_day == 7:
                week_day = 0
            
            # 获取当前是星期几
            current_week_day = self.base_time.weekday()
            if current_week_day == 0:  # 如果当天是星期一，则星期天是7天前
                current_week_day = 7
                
            if self.tp.tunit[0] == -1:
                self.tp.tunit[0] = self.base_time.year
            if self.tp.tunit[1] == -1:
                self.tp.tunit[1] = self.base_time.month
            if self.tp.tunit[2] == -1:
                self.tp.tunit[2] = self.base_time.day
                
            # 计算时间差
            day_gap = week_day - current_week_day
            if day_gap < 0:
                day_gap += 7
                
            # 如果用户没有指明是本周还是下周，这里默认是本周
            # 如果当前时间是周末，并且用户指定的是周一到周五，则默认是下周
            if self.is_first_time_solve_context:
                if day_gap == 0:
                    # 如果是同一天，则默认是下周
                    day_gap = 7
                    
            self.tp.tunit[2] += day_gap
            self.is_first_time_solve_context = False
            
        # 处理星期天的情况
        rule = r"(?<=[周星期])(天|日)"
        match = re.search(rule, self.time_expression)
        if match:
            # 获取当前是星期几
            current_week_day = self.base_time.weekday()
            if current_week_day == 0:  # 如果当天是星期一，则星期天是6天后
                current_week_day = 7
                
            if self.tp.tunit[0] == -1:
                self.tp.tunit[0] = self.base_time.year
            if self.tp.tunit[1] == -1:
                self.tp.tunit[1] = self.base_time.month
            if self.tp.tunit[2] == -1:
                self.tp.tunit[2] = self.base_time.day
                
            # 计算时间差
            day_gap = 7 - current_week_day
            
            # 如果用户没有指明是本周还是下周，这里默认是本周
            # 如果当前时间是周末，并且用户指定的是周一到周五，则默认是下周
            if self.is_first_time_solve_context:
                if day_gap == 0:
                    # 如果是同一天，则默认是下周
                    day_gap = 7
                    
            self.tp.tunit[2] += day_gap
            self.is_first_time_solve_context = False
            
        # 处理下周的情况
        rule = r"下(周|星期)"
        match = re.search(rule, self.time_expression)
        if match:
            # 获取当前是星期几
            current_week_day = self.base_time.weekday()
            if current_week_day == 0:  # 如果当天是星期一，则星期天是6天后
                current_week_day = 7
                
            if self.tp.tunit[0] == -1:
                self.tp.tunit[0] = self.base_time.year
            if self.tp.tunit[1] == -1:
                self.tp.tunit[1] = self.base_time.month
            if self.tp.tunit[2] == -1:
                self.tp.tunit[2] = self.base_time.day
                
            # 计算时间差
            day_gap = 7 - current_week_day + 7
            
            self.tp.tunit[2] += day_gap
            self.is_first_time_solve_context = False
            
        # 处理上周的情况
        rule = r"上(周|星期)"
        match = re.search(rule, self.time_expression)
        if match:
            # 获取当前是星期几
            current_week_day = self.base_time.weekday()
            if current_week_day == 0:  # 如果当天是星期一，则星期天是6天后
                current_week_day = 7
                
            if self.tp.tunit[0] == -1:
                self.tp.tunit[0] = self.base_time.year
            if self.tp.tunit[1] == -1:
                self.tp.tunit[1] = self.base_time.month
            if self.tp.tunit[2] == -1:
                self.tp.tunit[2] = self.base_time.day
                
            # 计算时间差
            day_gap = 7 - current_week_day - 7
            
            self.tp.tunit[2] += day_gap
            self.is_first_time_solve_context = False
            
        # 处理下下周的情况
        rule = r"下下(周|星期)"
        match = re.search(rule, self.time_expression)
        if match:
            # 获取当前是星期几
            current_week_day = self.base_time.weekday()
            if current_week_day == 0:  # 如果当天是星期一，则星期天是6天后
                current_week_day = 7
                
            if self.tp.tunit[0] == -1:
                self.tp.tunit[0] = self.base_time.year
            if self.tp.tunit[1] == -1:
                self.tp.tunit[1] = self.base_time.month
            if self.tp.tunit[2] == -1:
                self.tp.tunit[2] = self.base_time.day
                
            # 计算时间差
            day_gap = 7 - current_week_day + 14
            
            self.tp.tunit[2] += day_gap
            self.is_first_time_solve_context = False
            
        # 处理上上周的情况
        rule = r"上上(周|星期)"
        match = re.search(rule, self.time_expression)
        if match:
            # 获取当前是星期几
            current_week_day = self.base_time.weekday()
            if current_week_day == 0:  # 如果当天是星期一，则星期天是6天后
                current_week_day = 7
                
            if self.tp.tunit[0] == -1:
                self.tp.tunit[0] = self.base_time.year
            if self.tp.tunit[1] == -1:
                self.tp.tunit[1] = self.base_time.month
            if self.tp.tunit[2] == -1:
                self.tp.tunit[2] = self.base_time.day
                
            # 计算时间差
            day_gap = 7 - current_week_day - 14
            
            self.tp.tunit[2] += day_gap
            self.is_first_time_solve_context = False
    
    def prefer_future(self, index):
        """如果用户选择的时间是过去的时间，则将其转换为未来的时间"""
        # 1. 检查被检查的时间级别之前，是否没有更高级的已经确定的时间，如果有，则不进行处理
        for i in range(0, index):
            if self.tp.tunit[i] != -1:
                return
                
        # 2. 根据上下文，检查当前时间级别以及之前所有级别的时间
        time_arr = self.tp.tunit.copy()
        
        if self.tp.tunit[0] == -1:
            time_arr[0] = self.base_time.year
        if self.tp.tunit[1] == -1:
            time_arr[1] = self.base_time.month
        if self.tp.tunit[2] == -1:
            time_arr[2] = self.base_time.day
            
        # 将-1的部分设置为0
        for i in range(3, 6):
            if time_arr[i] == -1:
                time_arr[i] = 0
                
        # 构建当前时间
        cur = datetime.datetime(
            self.base_time.year, self.base_time.month, self.base_time.day,
            self.base_time.hour, self.base_time.minute, self.base_time.second
        )
        
        # 构建识别出的时间
        try:
            time_arr = [int(time_arr[i]) for i in range(0, 6)]
            cur_datetime = datetime.datetime(
                time_arr[0], time_arr[1], time_arr[2],
                time_arr[3], time_arr[4], time_arr[5]
            )
            
            # 如果识别出的时间小于当前时间，则将其设置为未来的时间
            if cur_datetime < cur:
                if index == 0:
                    time_arr[0] += 1
                elif index == 1:
                    time_arr[1] += 1
                    if time_arr[1] > 12:
                        time_arr[0] += 1
                        time_arr[1] = 1
                elif index == 2:
                    time_arr[2] += 1
                    if time_arr[2] > calendar.monthrange(time_arr[0], time_arr[1])[1]:
                        time_arr[1] += 1
                        time_arr[2] = 1
                        if time_arr[1] > 12:
                            time_arr[0] += 1
                            time_arr[1] = 1
                elif index == 3:
                    time_arr[3] += 24
                    if time_arr[3] >= 24:
                        time_arr[2] += 1
                        time_arr[3] -= 24
                        if time_arr[2] > calendar.monthrange(time_arr[0], time_arr[1])[1]:
                            time_arr[1] += 1
                            time_arr[2] = 1
                            if time_arr[1] > 12:
                                time_arr[0] += 1
                                time_arr[1] = 1
                elif index == 4:
                    time_arr[4] += 60
                    if time_arr[4] >= 60:
                        time_arr[3] += 1
                        time_arr[4] -= 60
                        if time_arr[3] >= 24:
                            time_arr[2] += 1
                            time_arr[3] -= 24
                            if time_arr[2] > calendar.monthrange(time_arr[0], time_arr[1])[1]:
                                time_arr[1] += 1
                                time_arr[2] = 1
                                if time_arr[1] > 12:
                                    time_arr[0] += 1
                                    time_arr[1] = 1
                elif index == 5:
                    time_arr[5] += 60
                    if time_arr[5] >= 60:
                        time_arr[4] += 1
                        time_arr[5] -= 60
                        if time_arr[4] >= 60:
                            time_arr[3] += 1
                            time_arr[4] -= 60
                            if time_arr[3] >= 24:
                                time_arr[2] += 1
                                time_arr[3] -= 24
                                if time_arr[2] > calendar.monthrange(time_arr[0], time_arr[1])[1]:
                                    time_arr[1] += 1
                                    time_arr[2] = 1
                                    if time_arr[1] > 12:
                                        time_arr[0] += 1
                                        time_arr[1] = 1
                                        
                self.tp.tunit[0] = time_arr[0]
                self.tp.tunit[1] = time_arr[1]
                self.tp.tunit[2] = time_arr[2]
                self.tp.tunit[3] = time_arr[3]
                self.tp.tunit[4] = time_arr[4]
                self.tp.tunit[5] = time_arr[5]
        except Exception as e:
            # 如果构建时间出错，则不进行处理
            pass
    
    def is_all_day(self):
        """判断是否是全天时间"""
        return self.is_all_day_time
    
    def get_time(self):
        """获取解析出的时间"""
        return self.time


class TimeNormalizer:
    """时间表达式识别的主要工具类"""
    def __init__(self, future=True):
        self.future = future  # 是否倾向于未来时间
        self.time_units = []  # 存储解析出的时间表达式
        
    def parse(self, time_string, base_time=None):
        """解析时间字符串"""
        if not base_time:
            base_time = datetime.datetime.now()
        elif isinstance(base_time, str):
            # 如果基准时间是字符串，则将其转换为datetime对象
            try:
                base_time = datetime.datetime.strptime(base_time, "%Y-%m-%d-%H-%M-%S")
            except ValueError:
                base_time = datetime.datetime.now()
                
        self.time_units = []
        
        # 对于中文数字，将其转换为阿拉伯数字
        time_string = self._normalize_chinese_numbers(time_string)
        
        # 提取所有可能的时间表达式
        time_expressions = self._extract_time_expressions(time_string)
        
        # 解析每个时间表达式
        for expression in time_expressions:
            time_unit = TimeUnit(expression, base_time)
            self.time_units.append(time_unit)
            
        return self.time_units
    
    def _normalize_chinese_numbers(self, text):
        """将中文数字转换为阿拉伯数字"""
        # 这里简化处理，只处理一些常见的中文数字
        chinese_numbers = {
            '零': '0', '一': '1', '二': '2', '三': '3', '四': '4',
            '五': '5', '六': '6', '七': '7', '八': '8', '九': '9',
            '十': '10', '百': '100', '千': '1000', '万': '10000'
        }
        
        for cn, num in chinese_numbers.items():
            text = text.replace(cn, num)
            
        return text
    
    def _extract_time_expressions(self, text):
        """提取所有可能的时间表达式"""
        # 这里简化处理，只提取一些常见的时间表达式
        # 实际应用中，可以使用更复杂的正则表达式或者机器学习方法
        expressions = []
        
        # 提取日期表达式
        date_patterns = [
                        r'\d{4}年\d{1,2}月\d{1,2}日',
            r'\d{4}-\d{1,2}-\d{1,2}',
            r'\d{4}/\d{1,2}/\d{1,2}',
            r'\d{4}\.\d{1,2}\.\d{1,2}',
            r'\d{1,2}月\d{1,2}日',
            r'今天|明天|后天|昨天|前天|大前天|大后天',
            r'上周|本周|下周|上上周|下下周',
            r'周[一二三四五六日天]|星期[一二三四五六日天]',
            r'上个月|这个月|下个月',
            r'去年|今年|明年|前年'
        ]
        
        # 提取时间表达式
        time_patterns = [
            r'\d{1,2}点\d{1,2}分\d{1,2}秒',
            r'\d{1,2}点\d{1,2}分',
            r'\d{1,2}点',
            r'\d{1,2}:\d{1,2}:\d{1,2}',
            r'\d{1,2}:\d{1,2}',
            r'凌晨|早上|早晨|早间|上午|中午|午间|下午|午后|晚上|傍晚|晚间|深夜',
            r'[零一二三四五六七八九十百千万\d]+点[零一二三四五六七八九十百千万\d]*分?[零一二三四五六七八九十百千万\d]*秒?'
        ]
        
        # 提取复合表达式
        for date_pattern in date_patterns:
            matches = re.finditer(date_pattern, text)
            for match in matches:
                expressions.append(match.group())
                
        for time_pattern in time_patterns:
            matches = re.finditer(time_pattern, text)
            for match in matches:
                expressions.append(match.group())
                
        # 如果没有提取到任何表达式，则将整个文本作为一个表达式
        if not expressions:
            expressions.append(text)
            
        return expressions
    
    def get_time_unit(self):
        """获取解析结果"""
        return self.time_units


def get_cron_date(date_string: str) -> str:
    """
    解析中文时间表达式，返回标准时间格式
    
    Args:
        date_string: 中文时间表达式，如"明天早上8点"
        
    Returns:
        标准时间格式的字符串，如"2023-05-20 08:00:00"
    """
    # 创建时间表达式识别器
    normalizer = TimeNormalizer()
    
    # 解析时间表达式
    time_units = normalizer.parse(date_string)
    
    # 如果没有解析出任何时间表达式，则返回当前时间
    if not time_units:
        return datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    # 获取第一个解析结果
    time_unit = time_units[0]
    
    # 获取解析出的时间
    parsed_time = time_unit.get_time()
    
    # 如果解析失败，则返回当前时间
    if not parsed_time:
        return datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    # 返回标准时间格式
    return parsed_time.strftime("%Y-%m-%d %H:%M:%S")


# 示例用法
if __name__ == "__main__":
    # 测试用例
    test_cases = [
        "明天早上8点",
        "下周一下午3点",
        "后天晚上8点半",
        "今天中午12点",
        "明年1月1日",
        "下下周一开会",
        "昨天上午，第八轮中美战略与经济对话气候变化问题特别联合会议召开"
    ]
    
    for test in test_cases:
        result = get_cron_date(test)
        print(f"输入: {test}")
        print(f"输出: {result}")
        print("-" * 30)