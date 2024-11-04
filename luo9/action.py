import value
import luo9
from plugins import api

async def poke_handle(target_id, user_id, group_id=''):
    # 群戳一戳
    if group_id != '':
        # 目标为bot本身
        if target_id == value.bot_id:
            message = await api.舔狗日记()
            await luo9.send_group_message(group_id, message, ignore=False)
    # 私聊戳一戳
    else:
        pass