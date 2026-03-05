import asyncio
import time
from luo9.api_manager import luo9
from . import state

def calculate_delay(message_list):
    delays = []
    typing_speed = 5

    for i in range(len(message_list) - 1):
        current_message = message_list[i]
        current_message_length = len(current_message)

        delay = current_message_length / typing_speed
        if delay > 4:
            delay = 4.0
        delays.append(delay)

    return delays

async def message_sender(group_id, user_id, message_list):
    delays = calculate_delay(message_list)
    for index, message in enumerate(message_list):
        if group_id != "":
            await luo9.send_group_message(group_id, message)
        else:
            await luo9.send_private_msg(user_id, message)
        if index < len(message_list) - 1:
            time.sleep(delays[index])

    state.sender_started = False

async def start_message_sender(group_id, user_id, message_list):
    asyncio.create_task(message_sender(group_id, user_id, message_list))
