import os
import asyncio
from luo9.api_manager import luo9
from luo9.message import GroupMessage
from utils.message_limit import MessageLimit
from config import get_value
from logger import Luo9Log

# 导入模块化组件
from .login import handle_login_request, cleanup_expired_logins, is_user_logged_in
from .query import perform_query, get_help_message, get_query_types_help
from .api import DeltaForceAPI
from luo9 import get_driver
from luo9 import get_task

value = get_value()
task = get_task()
driver = get_driver()
log = Luo9Log(__name__)

config = {
    'name': 'deltaforce',
    'describe': '三角洲行动API，提供二维码登录和游戏数据查询功能',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message', 'private_message'],
}

# 创建消息限制器
deltaforce_limit = MessageLimit('deltaforce')

# 创建数据目录
os.makedirs(f"{value.data_path}/plugins/{config['name']}", exist_ok=True)

@driver.on_startup
async def _():
    asyncio.create_task(cleanup_expired_logins())
    log.info("三角洲API：启动清理过期登录信息任务")

# 处理群消息
async def group_handle(message: GroupMessage):
    group_id = message.group_id
    user_id = message.user_id
    message_content = message.content
    
    if message_content == "三角洲登录" or message_content == "登录三角洲" and deltaforce_limit.check(3):
        deltaforce_limit.handle()
        await handle_login_request(group_id, user_id)
    
    elif message_content == "三角洲帮助":
        help_msg = get_help_message()
        await luo9.send_group_message(group_id, help_msg)
    
    # # elif message_content.startswith("三角洲查询") and is_user_logged_in(user_id):
    elif message_content.startswith("三角洲查询"):
        query = message_content.replace("三角洲查询", "").strip()
        if query:
            # await luo9.send_group_message(group_id, f"正在查询: {query}，请稍候...")
            result = await perform_query(query, user_id)
            await luo9.send_group_message(group_id, result)
        else:
            help_msg = get_query_types_help()
            await luo9.send_group_message(group_id, help_msg)

async def private_handle(message, user_id):
    message_content = message
    
    if message_content == "三角洲登录" and deltaforce_limit.check(30):
        deltaforce_limit.handle()
        await handle_login_request(None, user_id, is_private=True)
    
    elif message_content == "三角洲帮助":
        help_msg = get_help_message()
        await luo9.send_private_msg(user_id, help_msg)
    
    elif message_content.startswith("三角洲查询") and is_user_logged_in(user_id):
        query = message_content.replace("三角洲查询", "").strip()
        if query:
            await luo9.send_private_msg(user_id, f"正在查询: {query}，请稍候...")
            # 执行查询逻辑
            result = await perform_query(query, user_id)
            await luo9.send_private_msg(user_id, result)
        else:
            help_msg = get_query_types_help()
            await luo9.send_private_msg(user_id, help_msg)


# @task.on_schedule_task(trigger ='cron', second=0, minute=0, hour=6)
# async def _():
#     result = await perform_query(query, user_id)
#     await luo9.send_private_msg(user_id, result)
