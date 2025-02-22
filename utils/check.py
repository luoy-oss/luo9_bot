from config import get_value
value = get_value()

import re
import sqlite3
import os
import stat

from utils import ini_files as ini

async def data_path_check(group_id, qq):
    main_path = value.data_path
    
    # 群文件夹
    data_path = value.data_path + '/{group_id}/'.format(group_id=group_id)
    group_path = data_path
    if not os.path.exists(data_path):
        os.makedirs(data_path)
        os.chmod(data_path, stat.S_IRWXO)

    # 用户数据文件夹
    data_path = group_path + '/User/'
    user_path = data_path 
    if not os.path.exists(data_path):
        os.makedirs(data_path)
        os.chmod(data_path, stat.S_IRWXO)
    
    USER_DATA_PATH = user_path + '/{qq}.ini'.format(qq=qq)

    # 管理文件夹
    data_path = group_path + '/Admin/'
    admin_path = data_path
    if not os.path.exists(data_path):
        os.makedirs(data_path)
        os.chmod(data_path, stat.S_IRWXO)

    data_path = admin_path + '/priority/'
    admin_priority_path = data_path
    if not os.path.exists(data_path):
        os.makedirs(data_path)
        os.chmod(data_path, stat.S_IRWXO)
    
    ADMIN_PRIORITY_PATH = admin_priority_path + "/admin_authority.ini"
    ADMIN_FROZEN_PATH = admin_path + "/frozen.ini"


    path = {
        'USER_DATA_PATH': USER_DATA_PATH,
        'ADMIN_PRIORITY_PATH': ADMIN_PRIORITY_PATH,
        'ADMIN_FROZEN_PATH': ADMIN_FROZEN_PATH,
    }

    return path


async def frozen_check(group_id, qq, data_path=""):
    if data_path != "":
        if await ini.读配置项(data_path, "冻结账号", "{qq}".format(qq=qq), "否")  == "否":
            return False
        else:
            return True
    else:
        pass
    return True

async def interactiveState_check():
    print("interactiveState_check")
    return True

async def register_check(group_id, qq, data_path=""):
    if data_path != "":
        if await ini.读配置项(data_path, "注册", "是否注册", "否")  == "是":
            return True
        else:
            return False
    else:
        pass
    return False


def without_at(message, qq_id):
    return re.sub(f'\[CQ:at,qq={qq_id}\]\s*', '', message)

def at_check(message, qq_id):
    return without_at(message, qq_id) != message

async def duplicate_message_check(message, group_id, check_num):
    data_path = value.data_path + '/{group_id}/'.format(group_id=group_id)
    group_path = data_path
    if not os.path.exists(data_path):
        os.makedirs(data_path)
        os.chmod(data_path, stat.S_IRWXO)
        
    data_path = group_path + '/chat_record.db'
    record_file = data_path
    if os.path.isfile(data_path):
        conn = sqlite3.connect(data_path)
        cursor = conn.cursor()
        cursor.execute('select content from \'record\' order by id desc limit {check_num};'.format(check_num=check_num))
        records = cursor.fetchall()
        conn.commit()
        conn.close()
        if len(records) >= 3 and records[0] == records[1] == records[2]:
            return True
    return False