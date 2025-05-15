import requests
import asyncio
import time
import base64
import json
from luo9.api_manager import luo9
from logger import Luo9Log
from .api import DeltaForceAPI
from config import get_value

value = get_value()
log = Luo9Log(__name__)

# 存储登录状态的字典
login_status = {}


async def handle_login_request(group_id, user_id, is_private=False, config_name="deltaforce", is_send=True):
    """
    处理登录请求，获取二维码并发送给用户
    
    Args:
        group_id: 群组ID，私聊时为None
        user_id: 用户ID
        is_private: 是否为私聊消息
        config_name: 插件配置名称
        is_send: 是否默认发送玩家信息图片
    """
    # 获取二维码数据
    data = await DeltaForceAPI.get_data()
    
    if not data or "image" not in data:
        message = "获取二维码失败，请稍后再试"
        if is_private:
            await luo9.send_private_msg(user_id, message)
        else:
            await luo9.send_group_message(group_id, message)
        return
    
    qr_image_data = data["image"]
    qr_image_path = f"{value.data_path}/plugins/{config_name}/qr_{user_id}.png"
    
    try:
        # 解码base64并保存图片
        with open(qr_image_path, "wb") as f:
            f.write(base64.b64decode(qr_image_data))
        
        # 存储登录信息
        login_status[user_id] = {
            "qrSig": data["qrSig"],
            "token": data["token"],
            "loginSig": data["loginSig"],
            "cookie": data["cookie"],
            "access_token": "",
            "expires_in": "",
            "openid": "",
            "start_time": time.time(),
            "logged_in": False
        }
        
        # 发送二维码和提示消息
        if is_private:
            await luo9.send_private_msg(user_id, "请扫描二维码登录，二维码有效期为30秒")
            # 私聊发送图片
            await luo9.send_private_msg(user_id, f"[CQ:image,file=file:///{qr_image_path}]")
        else:
            await luo9.send_group_message(group_id, f"[CQ:at,qq={user_id}]\n请扫描二维码登录，二维码有效期为30秒")
            await luo9.send_group_image(group_id, qr_image_path)
        
        asyncio.create_task(check_login_status(group_id, user_id, is_private, is_send))
        return login_status[user_id]
    except Exception as e:
        message = f"处理二维码失败: {str(e)}"
        if is_private:
            await luo9.send_private_msg(user_id, message)
        else:
            await luo9.send_group_message(group_id, message)


async def check_login_status(group_id, user_id, is_private=False, is_send=True):
    """
    检查登录状态，定期查询API获取登录结果
    
    Args:
        group_id: 群组ID，私聊时为None
        user_id: 用户ID
        is_private: 是否为私聊消息
    """
    # 等待30秒，期间每3秒检查一次状态
    start_time = time.time()
    
    while time.time() - start_time < 30:
        if user_id not in login_status:
            print('用户ID不在登录状态中，说明已经被清理，直接退出')
            return
        
        # 获取登录信息
        login_info = login_status[user_id]
        
        # 检查登录状态
        response_json = await DeltaForceAPI.check_login_status(login_info)
        if response_json and response_json["code"] == -4:
            message = "为了保证你的账号安全，本次请求不支持图片识别或长按扫描二维码授权，请通过摄像头扫一扫重新授权登录。"
            if is_private:
                await luo9.send_private_msg(user_id, message)
            else:
                await luo9.send_group_message(group_id, message)
            
            if user_id in login_status:
                del login_status[user_id]
            return

        if response_json and response_json["code"] == 0:
            message = ""
            login_status[user_id]['cookie'] = response_json['data']['cookie']
            login_status[user_id]["logged_in"] = True

            message = "登录成功！现在您可以使用以下命令：\n"
            message += "1. 三角洲查询XXX - 进行查询\n"
            message += "2. 三角洲帮助 - 查看详细使用说明"

            try:
                response = requests.post(f"{DeltaForceAPI.BASE_URL}/qq/access",
                    data= {
                        'cookie': json.dumps(login_status[user_id]['cookie']),
                        'qq': str(user_id),
                    }
                )
                response_json = response.json()
                if response.status_code == 200:
                    # log.info(f"获取access_token成功: {response_json}")
                    login_status[user_id]["access_token"] = response_json["data"]["access_token"]
                    login_status[user_id]["expires_in"] = response_json["data"]["expires_in"]
                    login_status[user_id]["openid"] = response_json["data"]["openid"]
                else:
                    log.error(f"获取access_token失败: {response_json}")
                    return "获取access_token失败，请稍后再试" 
            except Exception as e:
                log.error(f"获取access_token异常: {str(e)}")
                return "获取access_token异常，请稍后再试"

            if is_private:
                await luo9.send_private_msg(user_id, message)
            else:
                await luo9.send_group_message(group_id, message)
                if is_send:
                    from .process import player_process
                    response_json = await DeltaForceAPI.perform_query("player", "", login_info)
                    if response_json and response_json["code"] == 0:
                        CQ_image = player_process(response_json)
                        await luo9.send_group_message(group_id, CQ_image)
                    
            return
        time.sleep(3)
    
    # 超时处理
    if user_id in login_status and not login_status[user_id].get("logged_in", False):
        message = "二维码已过期，请重新发送\"三角洲登录\"获取新的二维码"
        
        if is_private:
            await luo9.send_private_msg(user_id, message)
        else:
            await luo9.send_group_message(group_id, message)
        
        # 清理过期的登录信息
        if user_id in login_status:
            del login_status[user_id]


async def cleanup_expired_logins():
    """
    定期清理过期的登录信息
    """
    while True:
        current_time = time.time()
        expired_users = []
        
        for user_id, info in login_status.items():
            # 如果登录信息超过10分钟且未登录成功，则清理
            if not info.get("logged_in", False) and current_time - info.get("start_time", 0) > 1800:
                expired_users.append(user_id)
        
        # 清理过期的登录信息
        for user_id in expired_users:
            log.info(f"清理过期的登录信息: 用户ID {user_id}")
            del login_status[user_id]
        
        # 每10分钟检查一次
        await asyncio.sleep(600)


def is_user_logged_in(user_id):
    """
    检查用户是否已登录
    
    Args:
        user_id: 用户ID
        
    Returns:
        bool: 是否已登录
    """
    return user_id in login_status and login_status[user_id].get("logged_in", False)


def get_login_info(user_id):
    """
    获取用户的登录信息
    
    Args:
        user_id: 用户ID
        
    Returns:
        dict: 登录信息，未登录返回None
    """
    if is_user_logged_in(user_id):
        return login_status[user_id]
    return None