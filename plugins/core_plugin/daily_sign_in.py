from config import get_value
value = get_value()

import utils
from luo9.api_manager import luo9
import random

from utils import ini_files as ini
import datetime

async def sign_in(group_id, qq, path):
    USER_DATA_PATH = path['USER_DATA_PATH']
    ADMIN_FROZEN_PATH = path['ADMIN_FROZEN_PATH']

    point = 0
    total_point = 0
    sign_in_count = 0
    if await utils.interactiveState_check():
        msg = '[CQ:at,qq={qq}]\n'.format(qq=qq)
        if not await utils.frozen_check(group_id, qq, ADMIN_FROZEN_PATH):
            if await utils.register_check(group_id, qq, USER_DATA_PATH):
                if not await sign_in_today(group_id, qq, USER_DATA_PATH):
                    point = random.randint(1, 10)
                    date = datetime.datetime.now().strftime('%Y年%m月%d日')

                    total_point = int(await ini.读配置项(USER_DATA_PATH, "签到", "总积分"))  + point
                    sign_in_count =  int(await ini.读配置项(USER_DATA_PATH, "签到", "签到次数")) + 1

                    msg +=   '恭喜你签到成功！\n发送‘查询’查看相关信息~ \n' 
                    msg +=  '获得积分 {point} 点\n'.format(point=point)
                    msg +=  '总积分 {total_point} 点\n'.format(total_point=total_point)
                    msg +=  '签到次数 {sign_in_count} 次'.format(sign_in_count=sign_in_count)

                    await ini.写配置项(USER_DATA_PATH, "签到", "总积分", str(total_point))
                    await ini.写配置项(USER_DATA_PATH, "签到", "签到次数", str(sign_in_count))
                    await ini.写配置项(USER_DATA_PATH, "签到", "签到时间", date)  
                else:
                    msg += '您已经签到过了哦~\n发送‘查询’查看相关信息~'
            else:
                msg += '你还没有注册哦，请先注册'
        else:
            msg += "已禁止(请联系管理人员)"

        await luo9.send_group_message(group_id, msg)
    else:
        pass


async def sign_in_today(group_id, qq, data_path=""):
    if data_path != "":
        formatted_date = datetime.datetime.now().strftime('%Y年%m月%d日')
        if await ini.读配置项(data_path, "签到", "签到时间")  == formatted_date:
            return True
        else:
            return False
    else:
        pass
    return True

async def query_data(group_id, qq, path):
    USER_DATA_PATH = path['USER_DATA_PATH']
    ADMIN_FROZEN_PATH = path['ADMIN_FROZEN_PATH']

    if await utils.interactiveState_check():
        msg = '[CQ:at,qq={qq}]\n'.format(qq=qq)
        if not await utils.frozen_check(group_id, qq, ADMIN_FROZEN_PATH):
            if await utils.register_check(group_id, qq, USER_DATA_PATH):
                total_point = int(await ini.读配置项(USER_DATA_PATH, "签到", "总积分"))
                sign_in_count = int(await ini.读配置项(USER_DATA_PATH, "签到", "签到次数"))
                sign_in_date = await ini.读配置项(USER_DATA_PATH, "签到", "签到时间")

                msg +=  '总积分 {total_point} 点\n'.format(total_point=total_point)
                msg +=  '签到次数 {sign_in_count} 次\n'.format(sign_in_count=sign_in_count)
                msg +=  '签到时间 {sign_in_date}'.format(sign_in_date=sign_in_date)
            else:
                msg += '你还没有注册哦，请先注册'
        else:
            msg += "已禁止(请联系管理人员)"

        await luo9.send_group_message(group_id, msg)
    else:
        pass


async def user_info(group_id, qq, path):
    USER_DATA_PATH = path['USER_DATA_PATH']
    ADMIN_FROZEN_PATH = path['ADMIN_FROZEN_PATH']

    if await utils.interactiveState_check():
        msg = '[CQ:at,qq={qq}]\n'.format(qq=qq)
        if not await utils.frozen_check(group_id, qq, ADMIN_FROZEN_PATH):
            if await utils.register_check(group_id, qq, USER_DATA_PATH):
                好感 = int(await ini.读配置项(USER_DATA_PATH, "个人信息", "好感"))
                IV = int(await ini.读配置项(USER_DATA_PATH, "个人信息", "IV"))
                IV最大值 = int(await ini.读配置项(USER_DATA_PATH, "数据最值", "IV最大值"))
                心情 = int(await ini.读配置项(USER_DATA_PATH, "个人信息", "心情"))
                心情最大值 = int(await ini.读配置项(USER_DATA_PATH, "数据最值", "心情最大值"))
                msg +=  '好感： {value}\n'.format(value=好感)
                msg +=  'IV： {value}/{value_max}\n'.format(value=IV, value_max=IV最大值)
                msg +=  '心情： {value}/{value_max}'.format(value=心情, value_max=心情最大值)

            else:
                msg += '你还没有注册哦，请先注册'
        else:
            msg += "已禁止(请联系管理人员)"

        await luo9.send_group_message(group_id, msg)
    else:
        pass