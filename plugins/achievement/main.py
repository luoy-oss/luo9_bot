import os
import stat
import sqlite3
import platform
import utils.check as check
from luo9.api_manager import luo9
from luo9.message import GroupMessage
from random import choices
from plugins.festival import FestivalCalendar
from .data_value import festival_achievement
from config import get_value
value = get_value()

config = {
    'name': 'achievement',
    'describe': '成就模块',
    'dependency': 'festival',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}

async def group_handle(message: GroupMessage):
    group_id = message.group_id
    user_id = message.user_id
    message = message.content
    path = await check.data_path_check(group_id, user_id)
    if check.at_check(message, value.bot_id):
        festival_reward = await festival_match(check.without_at(message, value.bot_id), group_id, user_id, path)
        if festival_reward['status'] is True:
            print(festival_reward['rewards'])
            pass

    if message == "我的成就":
        await get_achievement(group_id, user_id, path)


async def get_achievement(group_id, qq, path):
    USER_DATA_PATH = path['USER_DATA_PATH']

    if await check.interactiveState_check():
        msg = '[CQ:at,qq={qq}]\n'.format(qq=qq)
        if await check.register_check(group_id, qq, USER_DATA_PATH):
            msg += '———成就列表———\n'

            data_path = value.data_path + '/Achievement/'
            achieve_path = data_path
            if not os.path.exists(data_path):
                os.makedirs(data_path)
                if platform.system() != 'Windows':
                    os.chmod(data_path, stat.S_IRWXO)
                
            data_path = achieve_path + '/achievements.db'
            record_file = data_path

            conn = sqlite3.connect(record_file)
            cursor = conn.cursor()

            cursor.execute('''
            CREATE TABLE IF NOT EXISTS '{user_id}' (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                achieve TEXT NOT NULL,
                datetime TEXT NOT NULL,
                remark TEXT NOT NULL
            )
            '''.format(user_id=qq))

            data = cursor.execute('''
                SELECT achieve, datetime, remark FROM '{user_id}'
                '''.format(user_id=qq)).fetchall()
    
            
            if len(data) > 0:
                for achieve, datetime, remark in data:
                    msg += achieve
                    if datetime != '':
                       msg += '-' + datetime
                    msg += '\n'
            else:
                pass

            conn.commit()
            conn.close()

        else:
            msg += '你还没有注册哦，请先注册'

        msg += '______________________'
        await luo9.send_group_message(group_id, msg)
    else:
        pass


async def festival_match(message, group_id, qq, path):
    festival = FestivalCalendar()
    if await check.interactiveState_check():
        msg = '[CQ:at,qq={qq}]\n'.format(qq=qq)
        element = festival.getCalendarDetail()
        if element['阳历节日'] != '' or element['农历节日'] != '':
            for festival, value in festival_achievement.items():
                orders = value['指令']
                replys = value['回复内容']
                rewards = value['节日奖励']
                if festival in element['阳历节日'] or festival in element['农历节日']:
                    if festival_orders_match(message, orders, festival):
                        reply = choices(replys)[0]
                        msg += reply
                        await luo9.send_group_message(group_id, msg)
                        return {'status': True, 'rewards': rewards}
                else:
                    if festival_orders_match(message, orders, festival) or message == f'{festival}快乐':
                        festival = element['阳历节日'] if element['阳历节日'] != '' else element['农历节日']
                        msg += f'不对哦，今天是{festival}\n艾特我，发送以下任意指令：\n{festival}快乐\n'
                        orders = festival_achievement[festival]['指令']
                        for order in orders:
                            msg += order + '\n'
                        msg += '领取节日礼物吧！'
                        await luo9.send_group_message(group_id, msg)
                        return {'status': False, 'rewards': []}
    else:
        pass
    return {'status': False, 'rewards': []}

def festival_orders_match(message, orders, festival):
    if message == f'{festival}快乐':
        return True
    else:
        for order in orders:
            if message == order:
                return True

    return False
