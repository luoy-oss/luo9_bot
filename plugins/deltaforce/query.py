import json
from logger import Luo9Log
from .api import DeltaForceAPI
from .login import get_login_info, is_user_logged_in
from .process import *

log = Luo9Log(__name__)

def _process():
    pass

# 支持的查询类型
QUERY_TYPES = {
    "游戏数据": ["data", data_process],
    "战绩": ["record", record_process],
    "物品信息": ["items", items_process],
    "配置文件": ["config", config_process],
    "玩家信息": ["player", player_process],
    "物品成交价": ["price", price_process],
    "玩家资产": ["assets", assets_process],
    "流水日志": ["logs", logs_process],
    "今日密码": ["password", password_process],
}


async def perform_query(query, user_id):
    """
    执行三角洲查询功能
    
    Args:
        query: 查询内容
        user_id: 用户ID
        
    Returns:
        str: 查询结果
    """

    print(f"查询内容: {query}， user_id: {user_id}")
    if not is_user_logged_in(user_id):
        return "您尚未登录，请先发送\"三角洲登录\"进行登录"
    else:
        pass
        # print('已登录, 开始查询')
    
    login_info = get_login_info(user_id)

    # 解析查询类型和内容
    query_type = None
    query_content = query
    query_process = _process
    
    for key, value in QUERY_TYPES.items():
        if query.startswith(key):
            query_type = value[0]
            query_content = query[len(key):].strip()
            query_process = value[1]
            break
    
    # 执行查询
    print(f"执行查询: {query_type}, {query_content}, {query_process}")
    response_json = await DeltaForceAPI.perform_query(query_type, query_content, login_info)

    if not response_json:
        return "查询失败，请稍后再试"
    
    if response_json["code"] == 0:
        return query_process(response_json)
    else:
        return f"查询失败: {response_json['msg']}"

def get_help_message():
    """
    获取帮助信息
    
    Returns:
        str: 帮助信息
    """
    help_msg = "三角洲行动API使用帮助：\n"
    help_msg += "当前为开发版本, 部分功能未实现(标x)\n"
    help_msg += "1. 发送【三角洲登录/登录三角洲】获取登录二维码\n"
    help_msg += "2. 登录成功后，可使用以下查询命令：\n"
    for query_type in QUERY_TYPES.keys():
        # help_msg += f"   - 三角洲查询{query_type} [内容]\n"
        if query_type in ["玩家信息", "今日密码"]:
            help_msg += f"   ➢ 三角洲查询{query_type}\n"
        else:
            help_msg += f"xxx 三角洲查询{query_type} [内容]\n"
    help_msg += "例如：三角洲查询玩家信息 某某玩家\n"
    help_msg += "或者：三角洲查询游戏数据"
    return help_msg


def get_query_types_help():
    """
    获取查询类型帮助信息
    
    Returns:
        str: 查询类型帮助信息
    """
    help_msg = "请输入查询内容，支持以下查询类型：\n"
    for query_type in QUERY_TYPES.keys():
        help_msg += f"- 三角洲查询{query_type} [内容]\n"
    help_msg += "例如：三角洲查询玩家信息 某某玩家"
    return help_msg