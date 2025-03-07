import os
import warnings

import yaml
from luo9.api_manager import luo9
from utils.message_limit import MessageLimit
from config import get_value
from luo9 import get_driver
from luo9 import get_task

from . import server as pc


task = get_task()
driver = get_driver()


value = get_value()

config = {
    'name': 'pc_status',
    'describe': '电脑状态检测',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
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

__pc_config = __load_config(value.plugin_path + '/pc_status', 'config.yaml')

pre_disk = None
task_handle = {
    "status": False,
    "cpu": False,
    "memory": False,
    "disk_write": False,
    "disk_read": False
}

@task.on_schedule_task(trigger ='interval', seconds=5)
async def pc_status_task():
    global pre_disk
    global task_handle
    data, pre_disk = pc.get_system_info(pre_disk)
    response = pc.check_alert_conditions(data) 
    new_alert = False
    alert_msg = ""
    ok_msg = ""

    metrics = {
        "cpu": pc.get_cpu_status,
        "memory": pc.get_memory_status,
        "disk_write": pc.get_disk_write_status,
        "disk_read": pc.get_disk_read_status
    }
    is_ok = True
    for metric, status_func in metrics.items():
        if status_func(response) == "alert":
            is_ok = False
            if not task_handle['status']:
                # 有告警任务,调整定时任务间隔
                task_handle['status'] = True
                task.adjust_interval(pc_status_task, 'interval', seconds=2)
                break
    if is_ok and task_handle['status']:
            # 无告警任务,任务之前处于告警检测状态
            # 调整为默认定时任务间隔
            task_handle['status'] = False
            task.adjust_interval(pc_status_task, 'interval', seconds=5)

    for metric, status_func in metrics.items():
        if status_func(response) == "alert":
            if not task_handle[metric]:
                task_handle[metric] = True
                new_alert = True
            alert_msg += "\n" + __pc_config['alert_message'][metric]
        else:
            if task_handle[metric]:
                task_handle[metric] = False
                ok_msg += "\n" + __pc_config['ok_message'][metric]

    msg = ""
    if alert_msg and ok_msg:
        msg += f"状态监测:{alert_msg}\n"
        msg += f"已就绪:{ok_msg}"
    elif new_alert and alert_msg:
        msg = f"状态监测:{alert_msg}"
    elif ok_msg:
        msg = f"已就绪:{ok_msg}"
    if msg:
        # print(msg)
        await message_push(msg)
    
pc_status_limit = MessageLimit("pc_status_limit")
async def group_handle(message, group_id, user_id):
    if message == "status" and pc_status_limit.check(5):
        pc_status_limit.handle()
        global pre_disk
        data = pc.get_current_status(pre_disk)
        msg = ""
        msg += "CPU使用率: {:.2f}%\n".format(data['cpu']['total'])
        msg += "内存使用率: {:.2f}%\n".format(data['memory']['percent'])
        msg += "磁盘写入速率: {}\n".format(data['disk']['write_rate'])
        msg += "磁盘读取速率: {}".format(data['disk']['read_rate'])

        await luo9.send_group_message(group_id, msg)
    pass

async def message_push(msg):
    if __pc_config['group_list']:
        for group_id in __pc_config['group_list']:
            await luo9.send_group_message(group_id, msg)
    if __pc_config['private_list']:
        for user_id in __pc_config['private_list']:
            await luo9.send_private_msg(user_id, msg)