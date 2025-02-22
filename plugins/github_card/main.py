import re
import requests
from luo9.api_manager import luo9
import utils.download_img as uimg
from utils.message_limit import MessageLimit
from config import get_value
value = get_value()

config = {
    'name': 'github_card',
    'describe': 'github链接解析为图片',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}


github_card_limit = MessageLimit('github_card')

async def group_handle(message, group_id, user_id):
    if "github.com" in message and github_card_limit.check(10):
        github_card_limit.handle()
        pattern = r".*?(?=github.com)"
        message = re.sub(pattern, '', message)
        github_card = await get_github_reposity_information(message)

        save_path = f"{value.data_path}/plugins/{config['name']}/{github_card['user_name']}.{github_card['repo_name']}.jpg"
        await uimg.download_image_if_needed(message, github_card['image_url'], save_path)
        await luo9.send_group_image(group_id, save_path)

async def get_github_reposity_information(url: str) -> str:
    # image_url = f"https://github.html.zone/{UserName}/{RepoName}"

    try:
        UserName, RepoName = url.replace("github.com/", "").split("/")
        image_url = f"https://opengraph.githubassets.com/githubcard/{UserName}/{RepoName}"
    except:
        UserName, RepoName = 'none', 'none'        
        image_url = f"https://opengraph.githubassets.com/githubcard/"

    return {'image_url': image_url, 'user_name': UserName, 'repo_name': RepoName}

