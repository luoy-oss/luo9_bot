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

class Message:
    def __init__(self):
        self.content = ""
        self.user_id = ""
        self.user_name = ""
        self.time = ""

    def handle(self, message_objects: dict):
        self.content = message_objects['message']
        self.user_id = message_objects['user_id']
        self.user_name = message_objects['sender']['nickname']
        self.time = message_objects['time']
        pass

class GroupMessage(Message):
    def __init__(self):
        super().__init__()
        self.group_id = ""
    
    def handle(self, message_objects: dict):
        super().handle(message_objects)
        self.group_id = message_objects['group_id']


class PrivateMessage(Message):
    def __init__(self):
        super().__init__()

    def handle(self, message_objects: dict):
        super().handle(message_objects)
        

__all__ = [Message, GroupMessage, PrivateMessage]
