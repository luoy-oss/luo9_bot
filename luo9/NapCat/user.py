import requests
from config import get_value
value = get_value()

from logger import Luo9Log
log = Luo9Log(__name__)


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
        log.info("Message sent successfully")
    else:
        log.warning("Failed to send message")
        log.warning(response.status_code, response.text)
 