from config import get_value
value = get_value()

import json
from logger import Luo9Log
from .api import DeltaForceAPI
log = Luo9Log(__name__)

def data_process(response_json):
    return "请等待后续更新"

def record_process(response_json):
    return "请等待后续更新"

def items_process(response_json):
    return "请等待后续更新"

def config_process(response_json):
    return "请等待后续更新"

def player_process(response_json):
    '''
    处理玩家信息
    {
    'code': 0,
    'msg': '获取成功',
    'data': {
        'player': {
            'picurl': '42010040094',    # 角色ID
            'charac_name': '是洛大人哟'    # 角色名
        },
        'game': {
            'result': 0,                # 
            'error_info': 0,            # 
            'rankpoint': '2092',        # 烽火地带总积分
            'tdmrankpoint': '3604',     # 全面战场总积分
            'soltotalfght': '313',      # 烽火地带总对局数
            'solttotalescape': '130',   # 烽火地带撤离成功场数
            'solduration': '224139',    # 烽火地带总时长（min）
            'soltotalkill': '177',      # 烽火地带击败干员数
            'solescaperatio': '41%',    # 烽火地带撤离率
            'avgkillperminute': '133',  # 分均击杀(需/100)
            'tdmduration': '1749',      # 全面战场总时长（min）
            'tdmsuccessratio': '31%',   # 全面战场胜率
            'tdmtotalfight': '86',      # 全面战场总对局数
            'totalwin': '27',           # 全面战场胜场
            'tdmtotalkill': 732         # 全面战场总击杀
        },
        'coin': 0,                  # 三角币
        'tickets': 240,             # 三角券
        'money': 8056968            # 哈夫币
        }
    }
    or 
    {'player': [], 'game': [], 'coin': 0, 'tickets': 0, 'money': 0}
    '''
    image = ""
    data = response_json['data']
    for key in data['game']:
        if key not in ['solescaperatio', 'tdmsuccessratio']:
            data['game'][key] = int(data['game'][key])  

    bg_img = None

    # 空数据时:    {'player': [], 'game': [], 'coin': 0, 'tickets': 0, 'money': 0}
    from .image import create_game_stats_image
    if not data['player']:
        pic_id = "none"
        image_path = f"{value.data_path}/plugins/deltaforce/{pic_id}.png"
        data = {'player': {'picurl': '42010040094', 'charac_name': ''}}
        create_game_stats_image(data, background_image_path=bg_img, output_path=image_path)
        image = f"[CQ:image,file={image_path}]"
    else:
        # 根据总场次和总撤离场次计算撤离率
        if data['game']['soltotalfght'] > 0:
            data['game']['solescaperatio'] = f"{data['game']['solttotalescape'] / data['game']['soltotalfght'] * 100:.1f}%"
        else:
            data['game']['solescaperatio'] = "0%"

        if data['game']['tdmtotalfight'] > 0:
            data['game']['tdmsuccessratio'] = f"{data['game']['totalwin'] / data['game']['tdmtotalfight'] * 100:.1f}%"
        else:
            data['game']['tdmsuccessratio'] = "0%"

        pic_id = data['player']['picurl']
        image_path = f"{value.data_path}/plugins/deltaforce/{pic_id}.png"
        create_game_stats_image(data, background_image_path=bg_img, output_path=image_path)
        image = f"[CQ:image,file={image_path}]"
    return image

def price_process(response_json):
    return "请等待后续更新"

def assets_process(response_json):
    return "请等待后续更新"

def logs_process(response_json):
    return "请等待后续更新"

def password_process(response_json):
    msg = ""
    msg += f"零号大坝: {response_json['data']['零号大坝']}"
    msg += f"\n长弓溪谷: {response_json['data']['长弓溪谷']}"
    msg += f"\n巴克什: {response_json['data']['巴克什']}"
    msg += f"\n航天基地: {response_json['data']['航天基地']}"
    return msg
