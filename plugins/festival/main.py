from datetime import datetime, timezone
from decimal import Decimal
from .date_value import *
from .element import Element
import re

__all__ = ['FestivalCalendar']

class FestivalCalendar:
    def __init__(self):
        self.elements = []
        self.cache = {}
        self.__length = 0  # 公历当月天数
        self.__firstWeek = 0  # 公历当月1日为星期几

    def getCalendarDetail(self):
        date = datetime.now()
        year = date.year
        month = date.month - 1
        day = date.day

        cacheKey = f'{year}-{month}'
        lunarCalendarUtil = FestivalCalendar()
        lunarCalendarUtil.calendar(year, month)
        self.cache[cacheKey] = lunarCalendarUtil

        element = lunarCalendarUtil.elements[int(day) - 1]

        element_dict = {
            '阳历': {
                '年': element.sYear,
                '月': element.sMonth,
                '日': element.sDay
            },
            '星期': element.week,
            '阴历': {
                '年': element.cYear,
                '月': element.cMonth,
                '日': element.cDay
            },
            '假期': True if element.isHoliday else False,
            '阳历节日': element.solarFestival,
            '农历节日': element.lunarFestival,
            '节气': element.solarTerms,

        }
        return element_dict

    def calendar(self, y: int, m: int):
        sDObj = None
        lDObj = None
        lL = False
        lD2 = None
        lY = lM = None
        lD = 1
        lX = 0
        tmp1 = None
        tmp2 = None
        lM2 = None
        lY2 = None
        tmp3 = None
        dayglus = None
        bsg = None
        xs = None
        xs1 = None
        fs = None
        fs1 = None
        cs = None
        cs1 = None
        cY = None  # 年柱
        cM = None  # 月柱
        cD = None  # 日柱
        lDPOS = [None, None, None]
        n = 0
        firstLM = 0
        dateString = f'{y}-{m + 1}-1'
        sDObj = datetime.strptime(dateString, '%Y-%m-%d')
        self.__length = self.__solarDays(y, m)
        self.__firstWeek = sDObj.weekday() + 1
        # 年柱 1900年立春后为庚子年(60进制36)
        if m < 2:
            cY = self.cyclical(y - 1900 + 36 - 1)
        else:
            cY = self.cyclical(y - 1900 + 36)

        term2 = self.sTerm(y, 2)  # 立春日期

        # 月柱 1900年1月小寒以前为 丙子月(60进制12)
        firstNode = self.sTerm(y, m * 2)  # 返回当月「节」为几日开始
        cM = self.cyclical((y - 1900) * 12 + m + 12)

        lM2 = (y - 1900) * 12 + m + 12

        # 当月一日与 1900/1/1 相差天数
        # 1900/1/1与 1970/1/1 相差25567日, 1900/1/1 日柱为甲戌日(60进制10)
        dateString = f'{y}-{m + 1}-1  00:00:00'
        date = datetime(y, m + 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        dayCyclical = date.timestamp() / 86400 + 25567 + 10

        df3 = datetime.strptime  # 简化日期解析

        n = 0
        i = 0
        for i in range(self.__length):
            # if i == 18:
            #     b = 5  # 这部分代码的作用不太明确，保留待确认
            if lD > lX:
                sDObj = datetime(y, m + 1, i + 1, 0, 0, 0, tzinfo=timezone.utc)  # 当月一日日期
                # df3(f'{y}-{m+1}-{i+1} 00:00:00', "%Y-%m-%d %H:%M:%S")
                lDObj = self.lunar_from_gregorian(sDObj)  # 农历
                lY = int(lDObj['year'])  # 农历年
                lM = int(lDObj['month'])  # 农历月
                lD = int(lDObj['day'])  # 农历日
                lL = bool(lDObj['isLeap'])  # 农历是否闰月
                lX = self.leap_days(lY) if lL else self.month_days(lY, lM)
                if n == 0:
                    firstLM = lM

                lDPOS[n] = i - lD + 1
                n += 1
            # ///////////////该弄这里了  2024年11月11日
            # 依节气调整二月分的年柱, 以立春为界
            if m == 1 and (i + 1) == term2:
                cY = self.cyclical(y - 1900 + 36)
                lY2 = (y - 1900 + 36)

            # 依节气月柱, 以「节」为界
            if i + 1 == firstNode:
                cM = self.cyclical((y - 1900) * 12 + m + 13)
                lM2 = (y - 1900) * 12 + m + 13
            # 日柱
            cD = self.cyclical(int(dayCyclical + i))
            lD2 = dayCyclical + i
            # print(y, m + 1, i + 1, (nStr1[(i + self.__firstWeek) % 7]), lY, lM, lD, lL, cY, cM, cD)
            element = Element(y, m + 1, i + 1, (nStr1[(i + self.__firstWeek) % 7]), lY, lM, lD, lL, cY, cM, cD)
            lD += 1
            element.cDay = self.cDay(element.lDay)
            paramterLy2 = -1 if lY2 == None else lY2 % 12
            paramterLm2 = -1 if lM2 == None else lM2 % 12
            paramterLd2 = -1 if lD2 == None else lD2 % 12
            paramterLy2b = -1 if lY2 == None else lY2 % 10
            paramterLy2c = int(-1 if lD2 == None else lD2 % 10)
            paramterLld = -1 if lD == None else lD - 1

            self.elements.append(element)
        # 2024年11月11日 到这里了
        # 节气
        tmp1 = self.sTerm(y, m * 2) - 1
        tmp2 = self.sTerm(y, m * 2 + 1) - 1
        self.elements[tmp1].solarTerms = solarTerm[m * 2]
        # 因为数组下标是从0开始,所以根据日期作为下标获取日期信息,则对应下标减一
        # elements.get(tmp2).solarTerms = solarTerm[m * 2 + 1];
        self.elements[tmp2 - 1].solarTerms = solarTerm[m * 2 + 1]
        if m == 3:
            self.elements[tmp1].color = "red"  # 清明颜色

        pattern = "^(\\d{2})(\\d{2})([\\s\\*])(.+)$"
        # 阳历节日
        for i in sFtv:
            matcher = re.match(pattern, i)
            if matcher != None:
                # 月份判断
                if int(matcher.group(1)) == m + 1:
                    self.elements[int(matcher.group(2)) - 1].solarFestival += matcher.group(4) + ""
                    # 放假日
                    if matcher.group(3) == '*':
                        self.elements[int(matcher.group(2)) - 1].isHoliday = True
        pattern = "^(\\d{2})(.{2})([\\s\\*])(.+)$"
        # 农历节日
        for i in lFtv:
            matcher = re.match(pattern, i)
            if matcher != None:
                tmp1 = int(matcher.group(1)) - firstLM
                if tmp1 == -11:
                    tmp1 = 1
                if 0 <= tmp1 < n:
                    tmp2 = int(lDPOS[tmp1]) + int(matcher.group(2)) - 1
                    if 0 <= tmp2 < self.__length:
                        self.elements[tmp2].lunarFestival += matcher.group(4)
                        # 放假日
                        if matcher.group(3) == '*':
                            self.elements[tmp2].isHoliday = True

        pass

    def cDay(self, d: int):
        s = ''
        if d == 10:
            s = "初十"
        elif d == 20:
            s = "二十"
        elif d == 30:
            s = "三十"
        else:
            s = nStr2[int(d // 10)]
            s += nStr1[int(d) % 10]
        return s

    def __solarDays(self, y: int, m: int):
        if m == 1:
            return 29 if ((y % 4 == 0) and (y % 100 != 0) or (y % 400 == 0)) else 28
        else:
            return solarMonth[m]

    def cyclical(self, num: int):
        return Gan[(num % 10)] + Zhi[(num % 12)]

    def sTerm(self, y: int, n: int):
        # 基准日期和时间（UTC时区）
        base_date = datetime(1900, 1, 6, 2, 5, 0, tzinfo=timezone.utc)
        base_timestamp = float(base_date.timestamp()) * 1000
        # 计算时间差
        time_diff = float(Decimal(31556925974.7) * Decimal(y - 1900) + Decimal(sTermInfo[n]) * Decimal(60000))
        timestamp = base_timestamp + time_diff
        # 转换为 datetime 对象(换算回去)
        result_date = datetime.fromtimestamp(int(timestamp / 1000.0), tz=timezone.utc)
        # 返回日期（从0开始）
        return result_date.day

    def leap_month(self, year):
        """判断农历年份是否为闰年，并返回闰月月份"""
        lm = lunarInfo[year - 1900] & 0xf
        return 0 if lm == 0xf else lm

    def year_days(self, year):
        """计算农历年份的总天数"""
        sum_days = 348
        i = 0x8000
        while i > 0x8:
            sum_days += 1 if (lunarInfo[year - 1900] & i) != 0 else 0
            i >>= 1
        return sum_days + self.leap_days(year)

    def leap_days(self, year):
        """计算农历年份闰月的天数"""
        lm = self.leap_month(year)
        if lm == 0:
            return 0
        return 30 if (lunarInfo[year - 1899] & 0xf) == 0xf else 29

    def month_days(self, year, month):
        """计算农历年份某个月的天数"""
        return 30 if (lunarInfo[year - 1900] & (0x10000 >> month)) != 0 else 29

    def is_leap_year(self, year):
        """判断是否为闰年"""
        return year % 4 == 0 and (year % 100 != 0 or year % 400 == 0)

    def days_in_month(self, year, month):
        """计算某个月的天数"""
        if month == 2 and self.is_leap_year(year):
            return solarMonth[1] + 1
        return solarMonth[month - 1]

    def lunar_from_gregorian(self, gregorian_date):
        """将公历日期转换为农历日期"""

        # 农历纪元开始日期
        # lunar_epoch = datetime(1900, 1, 31, 0, 0, 0, tzinfo=timezone.utc)

        # 计算公历日期距离农历纪元的天数
        # days_since_epoch = (gregorian_date - lunar_epoch).days

        # gregorian_date += timedelta(hours=8)
        date1 = float(gregorian_date.timestamp())
        date3 = float(datetime(1900, 1, 31, 0, 0, 0, tzinfo=timezone.utc).timestamp())
        offset = (date1 - date3) / 86400
        i = 1900
        while i < 2100 and offset > 0:
            temp = self.year_days(i)
            offset -= temp
            i += 1
        if offset < 0:
            offset += temp
            i -= 1
        year = i
        leap = self.leap_month(i)
        isLeap = False
        i = 1
        while i < 13 and offset > 0:
            # 闰月
            if (leap > 0 and i == leap + 1) and isLeap == False:
                i -= 1
                isLeap = True
                temp = self.leap_days(year)
            else:
                temp = self.month_days(year, i)
            # 解除闰月
            if isLeap == True and i == (leap + 1):
                isLeap = False
            offset -= temp
            i += 1
        if offset == 0 and leap > 0 and i == leap + 1:
            if isLeap:
                isLeap = False
            else:
                isLeap = True
                i -= 1
        if offset < 0:
            offset += temp
            i -= 1

        month = i
        day = offset + 1

        return {'year': year, 'month': month, 'day': day, "isLeap": isLeap}


if __name__ == '__main__':
    temp = FestivalCalendar()
    element = temp.getCalendarDetail()
    print(element)
