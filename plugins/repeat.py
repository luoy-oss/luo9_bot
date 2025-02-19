from config import get_value
value = get_value()

import utils
import luo9

repeate_message = ''

async def repeat(message, group_id):
    global repeate_message
    # 3次重复消息检测
    if not utils.at_check(message, value.bot_id) and message != repeate_message and await utils.duplicate_message_check(message, group_id, 3):
        repeate_message = message
        # await luo9.send_group_message(group_id, message)
        await luo9.send_group_message(group_id, message, ignore=False)
    else:
        pass