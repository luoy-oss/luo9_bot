import value
import luo9
import utils

from plugins import api
from utils.message_limit import MessageLimit

一言_limit = MessageLimit('一言')
情话_limit = MessageLimit('情话')
一言_网易云_limit = MessageLimit('一言_网易云')

async def poke_handle(target_id, user_id, group_id=''):
    # 群戳一戳
    if group_id != '':
        # 目标为bot本身
        if target_id == value.bot_id:
            global 一言_limit
            global 情话_limit
            global 一言_网易云_limit
            if utils.random_run(0.5):
                一言 = await api.一言()
                if 一言 != {} and 一言_limit.check(1):
                    # if utils.random_run(0.1):
                    #     radio_message = '{一言_content}。  ——来自《{一言_from}》'.format(一言_content=一言['content'], 一言_from=一言['from'])
                    #     await luo9.send_group_ai_radio(group_id, value.AI语音音色, radio_message)
                    # else:
                    message = '{一言_content}    ——来自《{一言_from}》'.format(一言_content=一言['content'], 一言_from=一言['from'])
                    await luo9.send_group_message(group_id, message, ignore=False)
            else:
                # 情话 = await api.情话()
                # if 情话 != '' and 情话_limit.check(1):
                #     await luo9.send_group_message(group_id, 情话, ignore=False)    
                一言_网易云 = await api.一言_网易云()
                if 一言_网易云 != '' and 一言_网易云_limit.check(1):
                    await luo9.send_group_message(group_id, 一言_网易云, ignore=False)         
    # 私聊戳一戳
    else:
        pass