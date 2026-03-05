from luo9.message import GroupMessage, PrivateMessage
from config import get_value
value = get_value()

from .conversation import (
    active_message,
    forget_conversation,
    restart_conversation,
    start_conversation,
    stop_conversation,
)

from .state import active_conversations

config = {
    "name": "ai_chat",
    "describe": "AI对话",
    "author": "drluo",
    "version": "1.0.0",
    "message_types": ["group_message", "private_message"],
}

async def group_handle(message: GroupMessage):
    content = message.content
    group_id = message.group_id
    user_id = message.user_id

    if content == "开!":
        return await start_conversation(group_id, user_id)
    elif content == "停!" or content == "关闭对话":
        return await stop_conversation(group_id, user_id)
    elif content == "遗忘对话":
        return await forget_conversation(group_id, user_id)
    elif content == "重启对话":
        return await restart_conversation(group_id, f'{user_id}')
    elif user_id in active_conversations:
        await active_message(message)
    else:
        return "对话未开启，请输入'开启对话'以开始聊天。"

async def private_handle(message: PrivateMessage):
    content = message.content
    user_id = message.user_id

    if content == "开!":
        return await start_conversation("", user_id)
    elif content == "停!" or content == "关闭对话":
        return await stop_conversation("", user_id)
    elif content == "遗忘对话":
        return await forget_conversation("", user_id)
    elif content == "重启对话":
        return await restart_conversation("", f'{user_id}')
    elif user_id in active_conversations:
        await active_message(message)
    else:
        return "对话未开启，请输入'开启对话'以开始聊天。"

