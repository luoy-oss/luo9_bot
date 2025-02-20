from config import get_value
value = get_value()

import utils
import plugins
import luo9

from utils.record import Record
from luo9 import action
from plugins import api
from utils.message_limit import MessageLimit

from luo9.plugin_manager import plugin_manager

'''
{
  "self_id": "BOT_ID",
  "user_id": "USER_ID",
  "time": 1730539514,
  "message_id": 1756106583,
  "message_seq": 1756106583,
  "real_id": 1756106583,
  "message_type": "group",
  "sender": {
    "user_id": "USER_ID",
    "nickname": "用户A",
    "card": "洛",
    "role": "owner"
  },
  "raw_message": "[CQ:at,qq=USER_ID] 你好世界[CQ:face,id=63]",
  "font": 14,
  "sub_type": "normal",
  "message": "[CQ:at,qq=USER_ID] 你好世界[CQ:face,id=63]",
  "message_format": "string",
  "post_type": "message",
  "group_id": GROUP_ID
}

{
  "self_id": "BOT_ID",
  "user_id": "USER_ID",
  "time": 1730539245,
  "message_id": 1846233392,
  "message_seq": 1846233392,
  "real_id": 1846233392,
  "message_type": "private",
  "sender": {
    "user_id": "USER_ID",
    "nickname": "用户A",
    "card": ""
  },
  "raw_message": "你好世界",
  "font": 14,
  "sub_type": "friend",
  "message": [
    {
      "type": "text",
      "data": {
        "text": "你好世界"
      }
    }
  ],
  "message_format": "array",
  "post_type": "message"
}
'''

@Record
async def message_handle(message_objects):
    if message_objects['message_type'] == 'group':
        message = message_objects['message']
        group_id = message_objects['group_id']
        user_id =  message_objects['user_id']
        
        await plugin_manager.handle_group_message(message, group_id, user_id)
    if message_objects['message_type'] == 'private':
        pass

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
        