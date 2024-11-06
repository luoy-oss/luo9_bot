import value
import luo9
import utils

from plugins import api

async def poke_handle(target_id, user_id, group_id=''):
    # 群戳一戳
    if group_id != '':
        # 目标为bot本身
        if target_id == value.bot_id:
            一言 = await api.一言()
            if 一言 != {}:
                # if utils.random_run(0.1):
                #     radio_message = '{一言_content}。  ——来自《{一言_from}》'.format(一言_content=一言['content'], 一言_from=一言['from'])
                #     await luo9.send_group_ai_radio(group_id, value.AI语音音色, radio_message)
                # else:
                message = '{一言_content}    ——来自《{一言_from}》'.format(一言_content=一言['content'], 一言_from=一言['from'])
                await luo9.send_group_message(group_id, message, ignore=False)
            
    # 私聊戳一戳
    else:
        pass