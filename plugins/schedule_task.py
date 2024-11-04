import time
import asyncio
import value
import luo9

from plugins import api


from apscheduler.schedulers.asyncio import AsyncIOScheduler

def schedule_run():
    print("定时任务初始化")
    scheduler = AsyncIOScheduler()

    scheduler.add_job(  
        摸鱼日历_task, 
        trigger ='cron', second=0, minute=0, hour=0)

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
    pass