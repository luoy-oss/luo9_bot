import value
import luo9
import plugins.api as api

async def poke_handle(target_id, user_id, group_id=''):
    # 群戳一戳
    if group_id != '':
        message = await api.舔狗日记()
        await luo9.send_group_message(group_id, message, ignore=False)
    # 私聊戳一戳
    else:
        pass