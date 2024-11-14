from .date_value import *


class Element:
    def __init__(self, sYear: int, sMonth: int, sDay: int, week: str, lYear: int, lMonth: int, lDay: int, isLeap: bool,
                 cYear: str, cMonth: str, cDay: str):
        self.isToday = False
        # 瓣句
        self.sYear = sYear  # 公元年4位数字
        self.sMonth = sMonth  # 公元月数字
        self.sDay = sDay  # 公元日数字
        self.week = week  # 星期, 1个中文
        # 农历
        self.lYear = lYear  # 公元年4位数字
        self.lMonth = lMonth  # 农历月数字
        self.lDay = lDay  # 农历日数字
        self.isLeap = isLeap  # 是否为农历闰月
        # 中文
        self.lMonthChinese = monthChinese[int(lMonth - 1)]
        self.lDayChinese = dayChinese[int(lDay - 1)]
        # 八字
        self.cYear = cYear  # 年柱, 2个中文
        self.cMonth = cMonth  # 月柱, 2个中文
        self.cDay = cDay  # 日柱, 2个中文

        self.isHoliday = False
        self.lunarFestival = ""  # 农历节日
        self.solarFestival = ""  # 公历节日
        self.solarTerms = ""  # 节气
