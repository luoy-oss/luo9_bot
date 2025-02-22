import random
import utils.check as check
from luo9.api_manager import luo9
from utils import ini_files as ini
from configparser import ConfigParser
from config import get_value
value = get_value()

async def register(group_id, qq, path):
    USER_DATA_PATH = path['USER_DATA_PATH']
    ADMIN_PRIORITY_PATH = path['ADMIN_PRIORITY_PATH']
    ADMIN_FROZEN_PATH = path['ADMIN_FROZEN_PATH']

    if await check.interactiveState_check():
        msg = '[CQ:at,qq={qq}]\n'.format(qq=qq)
        if not await check.frozen_check(group_id, qq, ADMIN_FROZEN_PATH):
            if not await check.register_check(group_id, qq, USER_DATA_PATH):
                config = ConfigParser()
                config["注册"] = {
                    "是否注册": "是"
                }
                config["个人信息"] = {
                    "好感": "1",
                    "IV": "70",
                    "心情": "0",
                }
                config["签到"] = {
                    "总积分": "0",
                    "签到次数": "0",
                    "签到时间": "你还没有签过到哦",
                }
                config["数据最值"] = {
                    "IV最大值": "70",
                    "心情最大值": "50",
                }
                await ini.配置项初始化(USER_DATA_PATH, config)

                config = ConfigParser()
                config["管理权限"] = {
                    "{qq}".format(qq=qq): "0"
                }
                await ini.配置项初始化(ADMIN_PRIORITY_PATH, config)


                config = ConfigParser()
                config["冻结账号"] = {
                    "{qq}".format(qq=qq): "否"
                }
                await ini.配置项初始化(ADMIN_FROZEN_PATH, config)


                point = random.randint(1, 10)
                msg += '注册成功！发送个人信息查看详情！\n'
                msg += '获得注册积分 {point} 点！'.format(point=point)

                await ini.写配置项(USER_DATA_PATH, "签到", "总积分", str(point))                
            else:
                msg += '你已经注册过了哦~\n发送个人信息查看详情！'

        else:
            msg += "已禁止(请联系管理人员)"

        await luo9.send_group_message(group_id, msg)
    else:
        pass