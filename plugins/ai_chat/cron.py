import asyncio
import json

from luo9.tasks import task
from luo9.api_manager import luo9
from config import get_value
value = get_value()

def parse_cron_expression(exp: str) -> dict:
    fields = exp.strip().split()
    if len(fields) not in (6, 7):
        raise ValueError(f"无效的Cron表达式: {exp}")
    
    replaced = [field.replace('?', '*') for field in fields]
    
    cron_kwargs = {
        'second': replaced[0],
        'minute': replaced[1],
        'hour': replaced[2],
        'day': replaced[3],
        'month': replaced[4],
        'day_of_week': replaced[5],
    }
    
    # 处理年份字段（如果存在）
    if len(fields) == 7:
        cron_kwargs['year'] = replaced[6]
    else:
        cron_kwargs['year'] = '*'
    
    return cron_kwargs

async def notice_message(messages, group_id):
    # await luo9.send_group_ai_record(group_id, value.AI语音音色, messages)
    await luo9.send_group_message(group_id, messages)

async def handle_cron_request(cron_req, group_id):
    print("申请定时：", cron_req)
    
    try:
        cron_data = json.loads(cron_req)
        cron_title = cron_data['cron']['title']
        cron_exp = cron_data['cron']['exp']
        messages = cron_data['cron']['content']
        cron_kwargs = parse_cron_expression(cron_exp)
        print("转换：", cron_kwargs)
        task.add_task(
            func=lambda: asyncio.run(notice_message(messages, group_id)),
            trigger='cron',
            **cron_kwargs
        )
    except (json.JSONDecodeError, KeyError, ValueError) as e:
        print(f"定时任务解析失败: {e}")
