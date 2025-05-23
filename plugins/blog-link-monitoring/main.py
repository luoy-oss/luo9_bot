import os
import re
import json
import yaml
import warnings
import requests

from config import get_value
from logger import Luo9Log

from luo9 import get_driver
from luo9 import get_task

value = get_value()
task = get_task()
driver = get_driver()
log = Luo9Log(__name__)

config = {
    'name': 'blog-link-monitoring',
    'describe': '主动向博客api进行网站状态监测请求',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': [],
}

def __load_config(path, file_name):
    config_path = os.path.join(path, file_name)
    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
    except FileNotFoundError:
        warnings.warn(f"在{path}目录中未找到文件{file_name} 使用默认样例config.(example).yaml配置\n请参考默认样例config.(example).yaml，在本插件目录中创建config.yaml进行配置")
        config_path = os.path.join(path, 'config.(example).yaml')
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
    return config

__config = __load_config(value.plugin_path + '/blog-link-monitoring', 'config.yaml')['monitoring']

if not __config['minutes'] or __config['minutes'] < 0:
    __config['minutes'] = 5

@driver.on_startup
async def _():
    log.info("blog-link-monitoring >>>> start")

# 每5分钟进行一次请求
@task.on_schedule_task(trigger ='interval', minutes=__config['minutes'], misfire_grace_time=16)
async def _api_task():
    await monitoring_api()

async def monitoring_api():
    api_url = f"{__config['api_url']}/api/batch-monitor"
    if __config.get('friend_link_url'):
        friend_link_url = __config['friend_link_url']
    else:
        friend_link_url = ''

    if __config.get('lists'):
        url_list = __config['lists']
    else:
        url_list = []

    if friend_link_url:
        params = {}
        response = requests.get(friend_link_url, params=params)
        if response.status_code == 200:
            response_json = response.json()
            log.info("github友链获取成功")
            
            urls = response_json
            for item in urls:
                issue_body = item['body']
                pattern = '```json\n([\s\S]*?)\n```'
                matchObj = re.search(pattern, issue_body)
                if matchObj and matchObj[1]:
                    link_data = json.loads(matchObj[1])
                    url_list.append(link_data['url'])
        else:
            log.warning(response.status_code, response.text)

    data = {
        'urls': url_list
    }

    response = requests.post(api_url, data=data)

    if response.status_code == 200:
        response_json = response.json()
        if response_json['success'] is True:
            log.info("友链监测成功")
        else:
            log.warning("友链监测失败")
    else:
        log.warning(response.status_code, response.text)
