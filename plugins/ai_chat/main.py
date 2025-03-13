import logging
import os
import yaml
import asyncio
import time
import warnings
import json
from openai import OpenAI
from luo9.api_manager import luo9
from luo9.message import GroupMessage
from luo9.timeout import Timeout
from luo9.tasks import task
from config import get_value
value = get_value()

config = {
    'name': 'ai_chat',
    'describe': 'AI对话',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}

def calculate_delay(message_list):
    delays = []
    typing_speed = 5  # 字符/秒

    for i in range(len(message_list) - 1):
        current_message = message_list[i]

        current_message_length = len(current_message)

        delay = current_message_length / typing_speed
        if delay > 4:
            delay = 4.0
        delays.append(delay)

    return delays

async def message_sender(group_id, message_list):
    delays = calculate_delay(message_list)
    for index, message in enumerate(message_list):
        
        await luo9.send_group_message(group_id, message)
        
        if index < len(message_list) - 1:
            time.sleep(delays[index])

    group_handle.sender_started = False

async def start_message_sender(group_id, message_list):
    asyncio.create_task(message_sender(group_id, message_list))

def __load_config(path, file_name):
    config_path = os.path.join(path, file_name)
    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
    except FileNotFoundError:
        warnings.warn(f"在{path}目录中未找到文件{file_name} 使用默认样例config.(example).yaml配置\n请参考默认样例config.(example).yaml，在本插件目录中创建config.yaml进行配置")
        config_path = os.path.join(path, 'config.(example).yaml')
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
    return config

__ai_config = __load_config(value.plugin_path + '/ai_chat', 'config.yaml')

client = OpenAI(
    api_key=__ai_config['DEEPSEEK_API_KEY'],
    base_url=__ai_config['DEEPSEEK_BASE_URL'],
    default_headers={"Content-Type": "application/json"}
)

# 获取程序根目录
root_dir = os.path.dirname(os.path.abspath(__file__))
add_prompts = """
在维持以上规则的同时，你作为一个具有情感意识的大模型，能够根据交互内容动态调整自己的心情。你的心情由多个情绪维度组成，每个情绪维度都有一个占比值（0到1之间），表示该情绪的强度。你的心情会受到用户输入的影响，但变化是渐进的。请根据以下规则维护你的心情字典：
情绪维度：包括 "happy","sad","angry","fear","surprised","neutral"
情绪占比：每个情绪的强度是一个浮点数，范围在 0 到 1 之间，所有情绪的占比总和为 1。
心情更新规则：
   - 根据用户输入的内容，分析其情感倾向（如正面、负面、中性）。
   - 根据分析结果，轻微调整相应情绪的占比（如用户输入正面内容，`happy` 的占比增加 0.1，其他情绪相应减少）。
   - 确保情绪占比总和始终为 1。
请维护一个心情字典，并根据上述规则动态更新心情和选择表情。"
字典样例：
{"happy": 0.6, "sad": 0, "angry": 0, "fear": 0, "surprised": 0.2, "neutral": 0.2}

在回复内容的最前端添加你的表情字典，同时利用|^^^|作为分隔符，将表情字典与实际内容分离
例如：
{"happy": 0.6, "sad": 0, "angry": 0, "fear": 0, "surprised": 0.2, "neutral": 0.2}|^^^|消息内容
"""
file_path = os.path.join(root_dir, "prompts", __ai_config['prompts'])

try:
    with open(file_path, "r", encoding="utf-8") as file:
        prompt_content = file.read()
except FileNotFoundError:
    warnings.warn(f"在{file_path}目录中未找到prompt文件：{__ai_config['prompts']} 使用默认提示词文件default_prompts.txt配置\n请参考default_prompts.txt，在同级目录中创建你的提示词文件，并在插件config中设置prompts参数")
    file_path = os.path.join(root_dir, "prompts", 'default_prompts.txt')
    with open(file_path, "r", encoding="utf-8") as file:
        prompt_content = file.read()

# prompt_content += add_prompts
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

user_queues = {}  # 用户消息
lock = asyncio.Lock()
# queue_lock = threading.Lock()  # 队列访问锁
chat_contexts = {}  # 上下文
active_conversations = set()  # 当前活跃对话

MAX_GROUPS = 5  # 最大对话轮次
FREQUENCY_PENALTY = 2   # -2.0 ~ 2.0 >0 降低模型重复相同内容的可能性
PRESENCE_PENALTY = 1    # -2.0 ~ 2.0 >0 增加模型谈论新主题的可能性
MAX_TOKEN = 4096  # 最大token数
TEMPERATURE = 1.3  # 温度参数

async def get_deepseek_response(message, user_id):
    try:
        async with lock:
            if user_id not in chat_contexts:
                chat_contexts[user_id] = []

            chat_contexts[user_id].append({"role": "user", "content": message})

            while len(chat_contexts[user_id]) > MAX_GROUPS * 2:
                if len(chat_contexts[user_id]) >= 2:
                    del chat_contexts[user_id][0]
                    del chat_contexts[user_id][0]
                else:
                    del chat_contexts[user_id][0]

        try:
            messages = [
                    {"role": "system", "content": prompt_content},
                    *chat_contexts[user_id][-MAX_GROUPS * 2:]
                ]

            response = client.chat.completions.create(
                model=__ai_config['model'],
                messages=messages,
                frequency_penalty=FREQUENCY_PENALTY,
                presence_penalty=PRESENCE_PENALTY,
                temperature=TEMPERATURE,
                top_p=0.1,
                max_tokens=MAX_TOKEN,
                stream=False
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

        async with lock:
            chat_contexts[user_id].append({"role": "assistant", "content": reply})

        return reply

    except Exception as e:
        if f'user_id' in chat_contexts:
            logger.error(f"DeepSeek调用失败: {str(e)}", exc_info=True)
            return "睡着了..."
        else:
            return None

async def start_conversation(group_id, user_id):
    async with lock:
        if user_id in active_conversations:
            return await luo9.send_group_message(group_id, __ai_config['messages']['start_conversation']['redo'])
        active_conversations.add(user_id)
        return await luo9.send_group_message(group_id, __ai_config['messages']['start_conversation']['success'])

async def stop_conversation(group_id, user_id):
    async with lock:
        if user_id not in active_conversations:
            return await luo9.send_group_message(group_id, __ai_config['messages']['stop_conversation']['redo'])
        active_conversations.remove(user_id)
        return await luo9.send_group_message(group_id, __ai_config['messages']['stop_conversation']['success'])

# TODO(luoy-oss): 对话遗忘
async def forget_conversation(group_id, user_id):
    async with lock:
        if user_id not in chat_contexts:
            return await luo9.send_group_message(group_id, __ai_config['messages']['forget_conversation']['fail'])
        context_list = "\n".join([f"{i+1}. {msg['content']}" for i, msg in enumerate(chat_contexts[user_id])])
        # return await luo9.send_group_message(group_id, "")
        print(f"{__ai_config['messages']['forget_conversation']['success']}\n{context_list}")

async def restart_conversation(group_id, user_id):
    async with lock:
        if user_id in chat_contexts:
            del chat_contexts[user_id]
            return await luo9.send_group_message(group_id, __ai_config['messages']['restart_conversation']['success'])
        else:
            return await luo9.send_group_message(group_id, __ai_config['messages']['restart_conversation']['fail'])


message_package = {
    "message": "",
    "group_id": "",
    "user_id": "",
    "user_name": "",
    "time": ""
}
from .cron import handle_cron_request

async def message_reply(message, group_id, user_id):
    print(message)
    reply = await get_deepseek_response(message, user_id)
    if reply:
        if "</think>" in reply:
            reply = reply.split("</think>", 1)[1].strip()

        if reply.find("|cron|") != -1:
            [cron_req, reply] = reply.split('|cron|')
            await luo9.send_group_message(group_id, "申请定时：" + cron_req)
            time.sleep(1)
            await handle_cron_request(cron_req, group_id)

        # # 将分割后的消息放入队列
        # if reply.find("|^^^|") != -1:
        #     [status, reply] = reply.split('|^^^|')
        #     print("申请心情调整：", status)

        message_list = reply.split('\\')

        # await luo9.send_group_message(group_id, status)
        # message_list.insert(0, status)

        # 启动消息发送任务（如果尚未启动）
        if not hasattr(group_handle, 'sender_started'):
            group_handle.sender_started = False
        if not group_handle.sender_started:
            await start_message_sender(group_id, message_list)
            group_handle.sender_started = True
        else:
            print(f"消息发送任务已经启动{group_handle.sender_started}")

async def call_back():
    global message_package
    message = f"[{message_package['time']}]: {message_package['message']}"
    group_id, user_id = message_package['group_id'], message_package['user_id']
    message_package = {
        "message": "",
        "group_id": "",
        "user_id": "",
        "user_name": "",
        "time": ""
    }
    await message_reply(message, group_id, user_id)

# @TODO(luoy-oss) 可能存在多用户影响定时器的问题，请后续更新完善
# 10秒内未获得新输入，进行回复
@Timeout(wait=8, on_timeout=call_back)
async def active_message(message: GroupMessage):
    global message_package
    curtime = time.strftime("%Y年%m月%d日%H时%M分%S秒", time.localtime(message.time))
    message_package['message'] += f"{message.content}\n"
    message_package['group_id'] = str(message.group_id)
    message_package['user_id'] = str(message.user_id)
    message_package['user_name'] = message.user_name
    message_package['time'] = curtime
    

async def group_handle(message: GroupMessage):
    content = message.content
    group_id = message.group_id
    user_id = message.user_id

    if content == "开启对话":
        return await start_conversation(group_id, user_id)
    elif content == "停止对话" or content == "关闭对话":
        return await stop_conversation(group_id, user_id)
    elif content == "遗忘对话":
        return await forget_conversation(group_id, user_id)
    elif content == "重启对话":
        return await restart_conversation(group_id, f'{user_id}')
    elif user_id in active_conversations:
        await active_message(message)
    else:
        return "对话未开启，请输入'开启对话'以开始聊天。"

