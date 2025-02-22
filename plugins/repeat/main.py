import utils.check as check
from luo9.api_manager import luo9
from config import get_value
value = get_value()

config = {
    'name': 'repeat',
    'describe': '复读机，对3词重复的内容进行复读',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}

repeate_message = ''

async def group_handle(message, group_id, user_id):
    global repeate_message
    # 3次重复消息检测
    if not check.at_check(message, value.bot_id) and message != repeate_message and await check.duplicate_message_check(message, group_id, 3):
        repeate_message = message
        # await luo9.send_group_message(group_id, message)
        await luo9.send_group_message(group_id, message, ignore=False)
    else:
        pass