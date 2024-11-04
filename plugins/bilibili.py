import urllib3
urllib3.disable_warnings()

import requests
from fake_useragent import UserAgent

async def live_check_with_liveid(live_id):
    headers = {'User-Agent': UserAgent().random}
    url = f"https://api.live.bilibili.com/room/v1/Room/get_info"
    params = {
        'room_id': live_id
    }
    response = requests.get(url, params=params,headers=headers, verify=False)
    live_json = response.json()
    status = 0
    if live_json['code'] == 0:
        # status 0：未开播 1：直播中 2：轮播中
        status = live_json['data']['live_status']

    return status
    

async def live_check_with_uid(uid):
    headers = {'User-Agent': UserAgent().random}
    url = f"https://api.live.bilibili.com/room/v1/Room/getRoomInfoOld"
    params = {
        "mid": uid
    }
    response = requests.get(url, params=params,headers=headers, verify=False)
    
    live_json = response.json()
    status = 0
    if live_json['code'] == 0:
        # status 0：未开播 1：直播中 2：轮播中
        status = live_json['data']['liveStatus']

    return status
    