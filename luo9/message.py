from utils.record import Record
from luo9.plugin_manager import plugin_manager
from config import get_value
value = get_value()

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

