import utils
import plugins
import luo9
import value

from plugins.chat import Record
from luo9 import action
from plugins import api

'''
{'self_id': 512166443, 'user_id': 2557657882, 'time': 1730539514, 'message_id': 1756106583, 'message_seq': 1756106583, 
'real_id': 1756106583, 
'message_type': 'group', 
'sender': {'user_id': 2557657882, 'nickname': '寂寞的根号二', 'card': '洛', 'role': 'owner'}, 
'raw_message': '[CQ:at,qq=2557657882] 你好世界[CQ:face,id=63]', 
'font': 14, 
'sub_type': 'normal', 
'message': '[CQ:at,qq=2557657882] 你好世界[CQ:face,id=63]', 
'message_format': 'string', 
'post_type': 'message', 
'group_id': 427124964}



{
'self_id': 512166443, 
'user_id': 2557657882, 
'time': 1730539245, 
'message_id': 1846233392, 
'message_seq': 1846233392, 
'real_id': 1846233392, 
'message_type': 'private', 
'sender': {'user_id': 2557657882, 'nickname': '寂寞的根号二', 'card': ''}, 
'raw_message': '你好世界', 
'font': 14, 
'sub_type': 'friend', 
'message': [{'type': 'text', 'data': {'text': '你好世界'}}], 
'message_format': 'array', 'post_type': 'message'}
'''

@Record
async def message_handle(message_objects):
    if message_objects['message_type'] == 'group':
        message = message_objects['message']
        group_id = message_objects['group_id']
        user_id =  message_objects['user_id']
        
        # if (group_id == 427124964 ): # or group_id == 706730261
        if group_id != 961949571:
            await group_message(message, group_id, user_id)
    if message_objects['message_type'] == 'private':
        pass

from datetime import datetime 
class MessageLimit:
    def __init__(self, tag):
        self.time_now = datetime.now()
        self.time_before = datetime(2000, 1, 1, 0, 0, 0, 0) 
        self.tag = tag

    def check(self, seconds):
        self.handle()
        if (self.time_now - self.time_before).seconds > seconds:
            self.time_before = datetime.now()
            return True
        else:
            return False
    def handle(self):
        self.time_now = datetime.now()
    def get_tag(self):
        return self.tag

摸鱼日历_limit = MessageLimit('摸鱼日历')
一言_limit = MessageLimit('一言')
情话_limit = MessageLimit('情话')
async def group_message(message, group_id, user_id):
    path = await utils.data_path_check(group_id, user_id)        
    if message == "签到" or message == "打卡":
        await plugins.sign_in(group_id, user_id, path)
    elif message == "查询":
        await plugins.query_data(group_id, user_id, path)
    elif message == "注册":
        await plugins.register(group_id, user_id, path)
    elif message == "个人信息":
        await plugins.user_info(group_id, user_id, path)

    elif utils.at_check(message, value.bot_id):
        global 摸鱼日历_limit
        global 一言_limit
        global 情话_limit
        if utils.without_at(message, value.bot_id) == '舔狗日记':
            msg = await api.舔狗日记()
            if not "妈的" in msg and not "你妈" in msg and not "他妈" in msg and not "去死" in msg and not "TT" in msg:
                await luo9.send_group_message(group_id, msg, ignore=False)
            else:
                print("舔狗日记：不文明用语屏蔽")
        # 60s回复屏蔽
        if 摸鱼日历_limit.check(60) and utils.without_at(message, value.bot_id) == '摸鱼日历':
            image_url = await api.摸鱼日历()
            await luo9.send_group_image(group_id, image_url)
        if 一言_limit.check(2) and utils.without_at(message, value.bot_id) == '一言':
            一言 = await api.一言()
            if 一言 != {}:
                message = '{一言_content}    ——来自《{一言_from}》'.format(一言_content=一言['content'], 一言_from=一言['from'])
                await luo9.send_group_message(group_id, message, ignore=False)
        if 情话_limit.check(2) and utils.without_at(message, value.bot_id) == '情话':
            情话 = await api.情话()
            if 情话 != '':
                await luo9.send_group_message(group_id, 情话, ignore=False)  
    else:
        # 非指令状态下进行复读
        await plugins.repeat(message, group_id)



    # if message == "一言":
    #     一言 = await api.一言()
    #     一言 = 一言['content']
    #     await luo9.send_group_message(group_id, 一言)


async def notice_handle(message_objects):
    if message_objects['notice_type'] == 'group_increase':
        group_id = message_objects['group_id']
        user_id = message_objects['user_id']
        # await luo9.send_group_message(group_id, f"欢迎新成员 [CQ:at,qq={user_id}] 加入群聊！")
    elif message_objects['notice_type'] == 'group_decrease':
        group_id = message_objects['group_id']
        user_id = message_objects['user_id']
        sub_type = message_objects['sub_type']
        # if sub_type == 'leave':
        #     await luo9.send_group_message(group_id, f"成员 [CQ:at,qq={user_id}] 已主动退出群聊。")
        # elif sub_type == 'kick':
        #     operator_id = message_objects['operator_id']
        #     await luo9.send_group_message(group_id, f"成员 [CQ:at,qq={user_id}] 被管理员 [CQ:at,qq={operator_id}] 踢出群聊。")
    elif message_objects['notice_type'] == 'notify':
        # 戳一戳(需要PacketServer才能进行回复)
        if message_objects['sub_type'] == 'poke':
            group_id = message_objects['group_id']
            target_id = message_objects['target_id']
            user_id = message_objects['user_id']
            if 'group_id' in message_objects:
                await action.poke_handle(target_id, user_id, group_id)            
            else:
                await action.poke_handle(target_id, user_id)

        pass
        