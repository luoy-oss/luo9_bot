import time
import asyncio
import value
import luo9
import value

from plugins import api
from plugins import bilibili
from utils import ini


from apscheduler.schedulers.asyncio import AsyncIOScheduler

def schedule_run():
    print("定时任务初始化")
    scheduler = AsyncIOScheduler()

    scheduler.add_job(  
        摸鱼日历_task, 
        trigger ='cron', second=0, minute=0, hour=6)

    scheduler.add_job(  
        B站直播检测_task, 
        trigger ='interval', minutes=1)

    scheduler.start()
    try:
        print("定时任务执行！")
        asyncio.get_event_loop().run_forever()
    except(KeyboardInterrupt, SystemExit):
        pass
    
async def 摸鱼日历_task():
    image_url = await api.摸鱼日历()
    for group_id in value.摸鱼日历_task_list:
        group_id = str(group_id)
        await luo9.send_group_image(group_id, image_url)
        await asyncio.sleep(1)    # 延迟1秒


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
