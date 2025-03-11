import requests
import random
import utils.check as check
from luo9.api_manager import luo9
from luo9.message import GroupMessage
from utils.message_limit import MessageLimit
from config import get_value
value = get_value()

config = {
    'name': 'api',
    'describe': '一言 情话 网易云 舔狗日记',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message','group_poke']
}

舔狗日记_limit = MessageLimit('舔狗日记')
一言_limit = MessageLimit('一言')
情话_limit = MessageLimit('情话')
一言_网易云_limit = MessageLimit('一言_网易云')
async def group_handle(message: GroupMessage):
    group_id = message.group_id
    message = message.content
    if check.at_check(message, value.bot_id):
        global 舔狗日记_limit
        global 一言_limit
        global 情话_limit
        if 舔狗日记_limit.check(2) and check.without_at(message, value.bot_id) == '舔狗日记':
            舔狗日记_limit.handle()
            api_msg = await 舔狗日记()
            if "妈的" not in api_msg and "你妈" not in api_msg and "他妈" not in api_msg and "去死" not in api_msg and "TT" not in api_msg:
                await luo9.send_group_message(group_id, api_msg, ignore=False)
            else:
                print("舔狗日记：不文明用语屏蔽")
        if 一言_limit.check(2) and check.without_at(message, value.bot_id) == '一言':
            一言_limit.handle()
            api_msg = await 一言()
            if api_msg != {}:
                message = '{一言_content}    ——来自《{一言_from}》'.format(一言_content=api_msg['content'], 一言_from=api_msg['from'])
                await luo9.send_group_message(group_id, message, ignore=False)
        if 情话_limit.check(2) and check.without_at(message, value.bot_id) == '情话':
            情话_limit.handle()
            api_msg = await 情话()
            if api_msg != '':
                await luo9.send_group_message(group_id, api_msg, ignore=False)  
        if 一言_网易云_limit.check(2) and check.without_at(message, value.bot_id) == '网易云':
            一言_网易云_limit.handle()
            api_msg = await 一言_网易云()
            if api_msg != '':
                await luo9.send_group_message(group_id, api_msg, ignore=False)  

async def group_poke_handle(target_id, user_id, group_id):
    # 目标为bot本身
    if target_id == value.bot_id:
        global 一言_limit
        global 情话_limit
        global 一言_网易云_limit
        choice = random.choice([1,2,3])
        if choice == 1:
            api_msg = await 一言()
            if api_msg != {} and 一言_limit.check(1):
                message = '{一言_content}    ——来自《{一言_from}》'.format(一言_content=api_msg['content'], 一言_from=api_msg['from'])
                await luo9.send_group_message(group_id, message, ignore=False)
        if choice == 2:
            api_msg = await 情话()
            if api_msg != '' and 情话_limit.check(1):
                await luo9.send_group_message(group_id, api_msg, ignore=False)    
        if choice == 3:
            api_msg = await 一言_网易云()
            if api_msg != '' and 一言_网易云_limit.check(1):
                await luo9.send_group_message(group_id, api_msg, ignore=False)         



async def 舔狗日记():
    url = "https://api.vvhan.com/api/text/dog"
    params = {
        'type': 'json'
    }
    
    response = requests.get(url, params=params)

    text = ''
    if response.status_code == 200:
        response_json = response.json()
        if response_json['success'] is True:
            print("api：舔狗日记获取成功")
            text = response_json['data']['content']
        else:
            print("api：舔狗日记获取失败 success: False")
    else:
        print("Failed api：舔狗日记")
        print(response.status_code, response.text)
    return text

async def 一言():
    url = "https://v1.hitokoto.cn"
    params = {
        'c': 'b'
    }
    
    response = requests.get(url, params=params)

    一言 = {}
    if response.status_code == 200:
        response_json = response.json()
        # if response_json['success'] == True:
        print("api：一言获取成功")
        一言 = {
            'creator':  response_json['creator'],
            'from':     response_json['from'],
            'content':  response_json['hitokoto'],
        }
        # else:
        #     print("api：一言获取失败 success: False")
    else:
        print("Failed api：一言")
        print(response.status_code, response.text)

    return 一言

async def 情话():
    url = "https://api.vvhan.com/api/text/love"
    params = {
        'type': 'json'
    }
    
    response = requests.get(url, params=params)

    情话 = ''
    if response.status_code == 200:
        response_json = response.json()
        if response_json['success'] is True:
            print("api：情话获取成功")
            情话 = response_json['data']['content']
        else:
            print("api：情话获取失败 success: False")
    else:
        print("Failed api：情话")
        print(response.status_code, response.text)
    return 情话

async def 一言_网易云():
    url = "https://v1.hitokoto.cn"
    params = {
        'c': 'j'
    }
    
    response = requests.get(url, params=params)

    一言_网易云 = {}
    if response.status_code == 200:
        response_json = response.json()
        # if response_json['success'] == True:
        print("api：一言_网易云获取成功")
        一言_网易云 = response_json['hitokoto']
        # else:
        #     print("api：一言_网易云获取失败 success: False")
    else:
        print("Failed api：一言_网易云")
        print(response.status_code, response.text)

    return 一言_网易云