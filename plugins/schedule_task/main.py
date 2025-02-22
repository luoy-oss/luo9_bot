
import asyncio
import requests
import urllib3
from luo9.api_manager import luo9
from utils import ini_files as ini
from config import get_value
from plugins.achievement.data_value import festival_achievement
from plugins.festival import FestivalCalendar
from luo9 import get_task
from fake_useragent import UserAgent
urllib3.disable_warnings()

task = get_task()
value = get_value()

config = {
    'name': 'schedule_task',
    'describe': '定时任务',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['']
}

@task.on_schedule_task(trigger ='interval', minutes=1)
async def B站直播检测_task():
    # status 0：未开播 1：直播中 2：轮播中
    status = await live_check_with_liveid(value.土豆直播间ID)
    live_flag = await ini.读配置项(f'{value.data_path}/bilibili_live.ini', f'{value.土豆直播间ID}', 'live', '0')
    if status == 1 and live_flag == '0':
        msg = "土豆开播啦！"
        live_flag = '1'
        await ini.写配置项(f'{value.data_path}/bilibili_live.ini', f'{value.土豆直播间ID}', 'live', live_flag)
        for group_id in value.B站直播检测推送列表:
            await luo9.send_group_message(group_id, msg)
            await asyncio.sleep(1)    # 延迟1秒
    if status == 0 and live_flag == '1':
        live_flag = '0'
        await ini.写配置项(f'{value.data_path}/bilibili_live.ini', f'{value.土豆直播间ID}', 'live', live_flag)  


async def live_check_with_liveid(live_id):
    headers = {'User-Agent': UserAgent().random}
    url = "https://api.live.bilibili.com/room/v1/Room/get_info"
    params = {
        'room_id': live_id
    }
    response = requests.get(url, params=params,headers=headers, verify=False)
    live_json = response.json()
    status = 0
    if live_json['code'] == 0:
        # status 0：未开播 1：直播中 2：轮播中
        status = live_json['data']['live_status']

    return status
    
# async def live_check_with_uid(uid):
#     headers = {'User-Agent': UserAgent().random}
#     url = "https://api.live.bilibili.com/room/v1/Room/getRoomInfoOld"
#     params = {
#         "mid": uid
#     }
#     response = requests.get(url, params=params,headers=headers, verify=False)
    
#     live_json = response.json()
#     status = 0
#     if live_json['code'] == 0:
#         # status 0：未开播 1：直播中 2：轮播中
#         status = live_json['data']['liveStatus']

#     return status
    

@task.on_schedule_task(trigger ='cron', second=0, minute=0, hour=6)
async def 节日检测_task():
    festival = FestivalCalendar()
    element = festival.getCalendarDetail()
    is_match = False
    if element['阳历节日'] != '' or element['农历节日'] != '':
        festival_today = element['阳历节日'] + element['农历节日']
        msg = ''
        for festival, temp in festival_achievement.items():
            if festival in element['阳历节日'] or festival in element['农历节日']:
                is_match = True
                msg += f'今天是{festival}\n艾特我，发送以下任意指令：\n{festival}快乐\n'
                orders = festival_achievement[festival]['指令']
                for order in orders:
                    msg += order + '\n'
                msg += '领取节日礼物吧！'
        
        if not is_match:
            msg += f'今天是{festival_today}。'

        for group_id in value.节日检测推送列表:
            await luo9.send_group_message(group_id, msg)
            await asyncio.sleep(1)    # 延迟1秒
    else:
        print("今天无节日")
    pass



