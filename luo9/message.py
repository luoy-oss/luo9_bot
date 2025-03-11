class Message:
    def __init__(self):
        self.content = ""
        self.user_id = ""
        self.time = ""

    def handle(self, message_objects: dict):
        self.content = message_objects['message']
        self.user_id = message_objects['user_id']
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
