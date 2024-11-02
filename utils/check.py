from utils import ini_files as ini
import value
import os, stat

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
    
    USER_DATA_PATH = user_path + '/{qq}.ini'.format(group_id=group_id,qq=qq)

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


