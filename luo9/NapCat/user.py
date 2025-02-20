from config import get_value
value = get_value()

from flask import Flask, request
import requests


# 发送群聊消息的函数
async def send_private_msg(user_id, message):
    url = f"{value.base_url}/send_private_msg"
    params = {
        "user_id": user_id,
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