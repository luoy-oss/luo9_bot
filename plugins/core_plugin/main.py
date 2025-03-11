from .daily_sign_in import sign_in, query_data, user_info
from .user_register import register
import utils.check as check
from luo9.message import GroupMessage

config = {
    'name': 'core_plugin',
    'describe': '签到注册模块',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}

async def group_handle(message: GroupMessage, group_id, user_id):
    message = message.content
    path = await check.data_path_check(group_id, user_id)
    if message == "签到" or message == "打卡":
        await sign_in(group_id, user_id, path)
    elif message == "查询":
        await query_data(group_id, user_id, path)
    elif message == "注册":
        await register(group_id, user_id, path)
    elif message == "个人信息":
        await user_info(group_id, user_id, path)

