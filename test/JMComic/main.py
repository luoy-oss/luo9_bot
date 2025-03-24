import re
from luo9.api_manager import luo9
from luo9.message import GroupMessage
import utils.download_img as uimg
from utils.message_limit import MessageLimit
from config import get_value
import jmcomic  # 导入此模块，需要先安装.
value = get_value()

config = {
    'name': 'JMComic',
    'describe': 'JMComic (禁漫天堂) 本子下载',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}


jm_limit = MessageLimit('JMComic')

async def group_handle(message: GroupMessage):
    group_id = message.group_id
    message = message.content
    if "jm" in message and jm_limit.check(3):
        jm_limit.handle()
        jmcomic.download_album('408778')
        await luo9.send_group_image(group_id, save_path)

async def get_JMComic_pdf(id: str) -> str:
    jmcomic.download_album('408778')  # 传入要下载的album的id，即可下载整个album到本地.
