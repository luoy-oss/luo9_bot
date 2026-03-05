import asyncio

user_queues = {}
lock = asyncio.Lock()
chat_contexts = {}
active_conversations = set()
message_package = {
    "message": "",
    "group_id": "",
    "user_id": "",
    "user_name": "",
    "time": "",
}
sender_started = False
