import value
import requests
from flask import Flask, request

async def send_group_message(group_id, message):
    if group_id in value.group_list:
        pass
    else:
        print(group_id,"不在",value.group_list,"中")
        print("消息阻断")
        return

    url = f"{value.base_url}/send_group_msg"
    params = {
        "group_id": group_id,
        "message": [message],
        "access_token": value.access_token
    }
    
    response = requests.get(url, params=params)
    # response = requests.post(url, data = params)

    if response.status_code == 200:
        print("Message sent successfully")
    else:
        print("Failed to send message")
        print(response.status_code, response.text)


async def send_group_at(group_id, qq):
    url = f"{value.base_url}/send_group_msg"
    params = {
        "group_id": group_id,
        "message": ['[CQ:at,qq={qq}]'.format(qq=qq)],
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)

    if response.status_code == 200:
        print("Message sent successfully")
    else:
        print("Failed to send message")
        print(response.status_code, response.text)

async def send_group_image(group_id, file):
    url = f"{value.base_url}/send_group_msg"
    params = {
        "group_id": group_id,
        "message": ['[CQ:image,file={file}]'.format(file=file)],
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)

    if response.status_code == 200:
        print("Message sent successfully")
    else:
        print("Failed to send message")
        print(response.status_code, response.text)

async def send_group_poke(group_id, target_id, user_id):
    # if str(user_id) != str(value.bot_id) and str(target_id) == str(value.bot_id):
    #         url = f"{value.base_url}/group_poke"
    #         params = {
    #             "group_id": group_id,
    #             "user_id": user_id,
    #             "access_token": value.access_token
    #         }
    #         print(params)
    #         response = requests.get(url, params=params)
    #         print(response.json())

    #         if response.status_code == 200:
    #             print("poke sent successfully")
    #         else:
    #             print("Failed to send poke")
    #             print(response.status_code, response.text)
    pass
