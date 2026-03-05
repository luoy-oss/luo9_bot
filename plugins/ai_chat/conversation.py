import time
from luo9.api_manager import luo9
from luo9.message import GroupMessage
from luo9.timeout import Timeout
from . import state
from .config_loader import ai_config, client, prompt_content, logger
from .cron import handle_cron_request
from .sender import start_message_sender

MAX_GROUPS = 5
FREQUENCY_PENALTY = 2
PRESENCE_PENALTY = 1
MAX_TOKEN = 4096
TEMPERATURE = 1.3

async def get_deepseek_response(message, curtime, user_id):
    try:
        async with state.lock:
            if user_id not in state.chat_contexts:
                state.chat_contexts[user_id] = []

            state.chat_contexts[user_id].append({"role": "user", "content": message})

            while len(state.chat_contexts[user_id]) > MAX_GROUPS * 2:
                if len(state.chat_contexts[user_id]) >= 2:
                    del state.chat_contexts[user_id][0]
                    del state.chat_contexts[user_id][0]
                else:
                    del state.chat_contexts[user_id][0]

        try:
            time_prompt = f"\n你的时间为：{curtime}\n"
            messages = [
                {"role": "system", "content": prompt_content + time_prompt},
                *state.chat_contexts[user_id][-MAX_GROUPS * 2 :],
            ]

            response = client.chat.completions.create(
                model=ai_config["model"],
                messages=messages,
                frequency_penalty=FREQUENCY_PENALTY,
                presence_penalty=PRESENCE_PENALTY,
                temperature=TEMPERATURE,
                top_p=0.1,
                max_tokens=MAX_TOKEN,
                stream=False,
            )
        except Exception as api_error:
            logger.error(f"API调用失败: {str(api_error)}")
            return "尝试...失败"

        if not response.choices:
            logger.error("API返回空choices: %s", response)
            return "为什么..获取不到回应了..."

        reply = response.choices[0].message.content
        logger.info(f"API响应 - 用户ID: {user_id}")
        logger.info(f"响应内容: {reply}")

        async with state.lock:
            state.chat_contexts[user_id].append({"role": "assistant", "content": reply})

        return reply

    except Exception as e:
        if f"user_id" in state.chat_contexts:
            logger.error(f"DeepSeek调用失败: {str(e)}", exc_info=True)
            return "睡着了..."
        else:
            return None

async def start_conversation(group_id, user_id):
    async with state.lock:
        if user_id in state.active_conversations:
            return await luo9.send_group_message(
                group_id, ai_config["messages"]["start_conversation"]["redo"]
            )
        state.active_conversations.add(user_id)
        return await luo9.send_group_message(
            group_id, ai_config["messages"]["start_conversation"]["success"]
        )

async def stop_conversation(group_id, user_id):
    async with state.lock:
        if user_id not in state.active_conversations:
            return await luo9.send_group_message(
                group_id, ai_config["messages"]["stop_conversation"]["redo"]
            )
        state.active_conversations.remove(user_id)
        return await luo9.send_group_message(
            group_id, ai_config["messages"]["stop_conversation"]["success"]
        )

async def forget_conversation(group_id, user_id):
    async with state.lock:
        if user_id not in state.chat_contexts:
            return await luo9.send_group_message(
                group_id, ai_config["messages"]["forget_conversation"]["fail"]
            )
        context_list = "\n".join(
            [f"{i+1}. {msg['content']}" for i, msg in enumerate(state.chat_contexts[user_id])]
        )
        print(f"{ai_config['messages']['forget_conversation']['success']}\n{context_list}")

async def restart_conversation(group_id, user_id):
    async with state.lock:
        if user_id in state.chat_contexts:
            del state.chat_contexts[user_id]
            return await luo9.send_group_message(
                group_id, ai_config["messages"]["restart_conversation"]["success"]
            )
        else:
            return await luo9.send_group_message(
                group_id, ai_config["messages"]["restart_conversation"]["fail"]
            )

async def message_reply(message, curtime, group_id, user_id):
    print(message)
    reply = await get_deepseek_response(message, curtime, user_id)
    if reply:
        if "</think>" in reply:
            reply = reply.split("</think>", 1)[1].strip()

        if reply.find("|cron|") != -1:
            [cron_req, reply] = reply.split("|cron|")
            await luo9.send_group_message(group_id, "申请定时：" + cron_req)
            time.sleep(1)
            await handle_cron_request(cron_req, group_id)

        message_list = reply.split("\\")

        if not state.sender_started:
            await start_message_sender(group_id, message_list)
            state.sender_started = True
        else:
            print(f"消息发送任务已经启动{state.sender_started}")

async def call_back():
    message = state.message_package["message"]
    curtime = state.message_package["time"]
    group_id = state.message_package["group_id"]
    user_id = state.message_package["user_id"]
    state.message_package = {
        "message": "",
        "group_id": "",
        "user_id": "",
        "user_name": "",
        "time": "",
    }
    await message_reply(message, curtime, group_id, user_id)

@Timeout(wait=8, on_timeout=call_back)
async def active_message(message: GroupMessage):
    if isinstance(message.time, str):
        timestamp = int(message.time)
    else:
        timestamp = message.time

    curtime = time.strftime("%Y年%m月%d日%H时%M分%S秒", time.localtime(timestamp))
    state.message_package["message"] += f"{message.content}\n"
    state.message_package["group_id"] = str(message.group_id)
    state.message_package["user_id"] = str(message.user_id)
    state.message_package["user_name"] = message.user_name
    state.message_package["time"] = curtime
