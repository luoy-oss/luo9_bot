import os
from config import get_value

value = get_value()


def get_data_path(config_name="deltaforce"):
    """
    获取插件数据目录路径
    
    Args:
        config_name: 插件配置名称
        
    Returns:
        str: 数据目录路径
    """
    return f"{value.data_path}/plugins/{config_name}"


def get_qr_image_path(user_id, config_name="deltaforce"):
    """
    获取用户二维码图片路径
    
    Args:
        user_id: 用户ID
        config_name: 插件配置名称
        
    Returns:
        str: 二维码图片路径
    """
    return f"{get_data_path(config_name)}/qr_{user_id}.png"


def ensure_data_directory(config_name="deltaforce"):
    """
    确保数据目录存在
    
    Args:
        config_name: 插件配置名称
    """
    os.makedirs(get_data_path(config_name), exist_ok=True)
