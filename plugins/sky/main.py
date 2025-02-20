config = {
    'name': 'sky',
    'describe': '光遇api，提供红石查询 每日任务查询',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}
from config import get_value
value = get_value()

import utils
from luo9.api_manager import luo9
import urllib.request
from utils.message_limit import MessageLimit

skyhs_limit = MessageLimit('skyhs')
skyrw_limit = MessageLimit('skyrw')
skyjl_limit = MessageLimit('skyjl')

repeate_message = ''

async def group_handle(message, group_id, user_id):
    if skyhs_limit.check(30) and (message == "sky红石" or message == "sky红石雨" or message == "sky黑石"):
        skyhs_limit.handle()
        img_url = 'https://api.zxz.ee/api/sky/?type=&lx=hs'
        save_path = f"{value.data_path}/plugins/{config['name']}/hs.jpg"
        await utils.download_image_if_needed(message, img_url, save_path)
        await luo9.send_group_image(group_id, save_path)

    if skyrw_limit.check(30) and (message == "sky每日任务" or message == "sky每日" or message == "sky任务"):
        skyrw_limit.handle()
        img_url = 'https://api.zxz.ee/api/sky/?type=&lx=rw'
        save_path = f"{value.data_path}/plugins/{config['name']}/rw.jpg"
        await utils.download_image_if_needed(message, img_url, save_path)
        await luo9.send_group_image(group_id, save_path)

    if skyjl_limit.check(30) and (message == "sky季节蜡烛" or message == "sky季蜡"):
        skyjl_limit.handle()
        img_url = 'https://api.zxz.ee/api/sky/?type=&lx=jl'
        save_path = f"{value.data_path}/plugins/{config['name']}/jl.jpg"
        await utils.download_image_if_needed(message, img_url, save_path)
        await luo9.send_group_image(group_id, save_path)

        # 任务_list = sky_api('rw')


async def sky_api(lx):
    url = f"https://api.zxz.ee/api/sky/?type=json&lx={lx}"
    params = {
        'type': 'json'
    }
    
    response = requests.get(url, params=params)

    text = ''
    if response.status_code == 200:
        response_json = response.json()
        if response_json['success'] == True:
            print("api：光遇红石获取成功")
            text = response_json['data']['content']
        else:
            print("api：舔狗日记获取失败 success: False")
    else:
        print("Failed api：舔狗日记")
        print(response.status_code, response.text)
    return text
