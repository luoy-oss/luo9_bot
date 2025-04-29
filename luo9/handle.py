from luo9.plugin_manager import plugin_manager
from utils.record import Record
from .message import GroupMessage, PrivateMessage
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
__message = GroupMessage()

# 群消息，私聊消息处理
# @Record
async def message_handle(message_objects):
    if message_objects['message_type'] == 'group':
        global message
        __message.handle(message_objects)
        
        await plugin_manager.handle_group_message(__message)
    if message_objects['message_type'] == 'private':
        pass

# 戳一戳消息处理
async def poke_handle(target_id, user_id, group_id=''):
    # 群戳一戳
    if group_id != '':
        await plugin_manager.handle_group_poke(str(target_id), str(user_id), str(group_id))
    # 私聊戳一戳
    else:
        pass

# 通知消息处理
async def notice_handle(message_objects):
    if message_objects['notice_type'] == 'group_increase':
        group_id = message_objects['group_id']
        user_id = message_objects['user_id']
        # await luo9.send_group_message(group_id, f"欢迎新成员 [CQ:at,qq={user_id}] 加入群聊！")
    elif message_objects['notice_type'] == 'group_decrease':
        group_id = message_objects['group_id']
        user_id = message_objects['user_id']
        # sub_type = message_objects['sub_type']
        # if sub_type == 'leave':
        #     await luo9.send_group_message(group_id, f"成员 [CQ:at,qq={user_id}] 已主动退出群聊。")
        # elif sub_type == 'kick':
        #     operator_id = message_objects['operator_id']
        #     await luo9.send_group_message(group_id, f"成员 [CQ:at,qq={user_id}] 被管理员 [CQ:at,qq={operator_id}] 踢出群聊。")
    elif message_objects['notice_type'] == 'notify':
        # 戳一戳(需要PacketServer才能进行回复)
        if message_objects['sub_type'] == 'poke':
            target_id = message_objects['target_id']
            user_id = message_objects['user_id']
            if 'group_id' in message_objects:
                group_id = message_objects['group_id']
                await poke_handle(target_id, user_id, group_id)
            else:
                await poke_handle(target_id, user_id)

        pass
        