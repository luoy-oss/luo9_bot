import requests

async def 舔狗日记():
    url = f"https://api.vvhan.com/api/text/dog"
    params = {
        'type': 'json'
    }
    
    response = requests.get(url, params=params)

    text = ''
    if response.status_code == 200:
        response_json = response.json()
        if response_json['success'] == True:
            print("api：舔狗日记获取成功")
            text = response_json['data']['content']
        else:
            print("api：舔狗日记获取失败 success: False")
    else:
        print("Failed api：舔狗日记")
        print(response.status_code, response.text)
    return text

async def 摸鱼日历():   
    url = f"https://api.vvhan.com/api/moyu"
    params = {
        'type': 'json'
    }
    
    response = requests.get(url, params=params)

    image_url = ''
    if response.status_code == 200:
        response_json = response.json()
        if response_json['success'] == True:
            print("api：摸鱼日历获取成功")
            image_url = response_json['url']
        else:
            print("api：摸鱼日历获取失败 success: False")
    else:
        print("Failed api：摸鱼日历")
        print(response.status_code, response.text)
    return image_url


async def 一言():
    url = f"https://api.vvhan.com/api/ian/dongman"
    params = {
        'type': 'json'
    }
    
    response = requests.get(url, params=params)

    一言 = {}
    if response.status_code == 200:
        response_json = response.json()
        if response_json['success'] == True:
            print("api：一言获取成功")
            一言 = {
                'creator':  response_json['data']['creator'],
                'from':     response_json['data']['form'],
                'content':  response_json['data']['content'],
            }
        else:
            print("api：一言获取失败 success: False")
    else:
        print("Failed api：一言")
        print(response.status_code, response.text)

    return 一言
