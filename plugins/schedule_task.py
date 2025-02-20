from config import get_value
value = get_value()

import time
import asyncio
from luo9.api_manager import luo9

from plugins import api
from plugins import bilibili
from utils import ini


from apscheduler.schedulers.asyncio import AsyncIOScheduler

def schedule_run():
    print("定时任务初始化")
    scheduler = AsyncIOScheduler()

    scheduler.add_job(  
        B站直播检测_task, 
        trigger ='interval', minutes=1)

    scheduler.add_job(  
        节日检测_task, 
        trigger ='cron', second=0, minute=0, hour=6)

    scheduler.start()
    try:
        print("定时任务执行！")
        asyncio.get_event_loop().run_forever()
    except(KeyboardInterrupt, SystemExit):
        pass
    
async def B站直播检测_task():
    # status 0：未开播 1：直播中 2：轮播中
    status = await bilibili.live_check_with_liveid(value.土豆直播间ID)
    live_flag = await ini.读配置项(f'{value.data_path}/bilibili_live.ini', f'{value.土豆直播间ID}', 'live', '0')
    if status == 1 and live_flag == '0':
        msg = "土豆开播啦！"
        live_flag = '1'
        await ini.写配置项(f'{value.data_path}/bilibili_live.ini', f'{value.土豆直播间ID}', 'live', live_flag)
        for group_id in value.B站直播检测_task_list:
            await luo9.send_group_message(group_id, msg, ignore=False)
            await asyncio.sleep(1)    # 延迟1秒
    if status == 0 and live_flag == '1':
        live_flag = '0'
        await ini.写配置项(f'{value.data_path}/bilibili_live.ini', f'{value.土豆直播间ID}', 'live', live_flag)  

from plugins.achievement.data_value import festival_achievement
from plugins.festival import FestivalCalendar

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

        for group_id in value.节日检测_task_list:
            await luo9.send_group_message(group_id, msg)
            await asyncio.sleep(1)    # 延迟1秒
    else:
        print("今天无节日")
    pass



