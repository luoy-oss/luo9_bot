import os
import re
import random
from luo9.api_manager import luo9
from luo9.message import GroupMessage
from config import get_value
from utils.message_limit import MessageLimit

from .battle import Battle
from .ranking import Ranking
from .inventory import Inventory
from .items import ItemManager
from .enhancement import Enhancement
from .database import Database
from .gif_generator import GifGenerator

value = get_value()

config = {
    'name': 'battle_arena',
    'describe': 'GIF格斗游戏，包含对战、排行榜、背包、物品和强化功能',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}

# 创建消息限制器
battle_limit = MessageLimit('battle_arena')

# 初始化各个模块
db = Database()
battle = Battle(db)
ranking = Ranking(db)
item_manager = ItemManager(db)
inventory = Inventory(db, item_manager)
enhancement = Enhancement(db, item_manager)
gif_generator = GifGenerator()

async def show_help(group_id, user_id, *args):
    """显示帮助信息"""
    help_text = (
        "【格斗游戏帮助】\n"
        "- 对战/挑战/格斗 @某人：向某人发起挑战\n"
        "- 排行榜：查看战力排行榜\n"
        "- 我的背包：查看自己的物品\n"
        "- 装备 [物品名]：装备指定物品\n"
        "- 卸下 [物品名]：卸下指定物品\n"
        "- 强化 [物品名]：强化指定物品\n"
        "- 商店：查看可购买的物品\n"
        "- 购买 [物品名]：购买指定物品\n"
        "- 我的状态：查看自己的属性\n"
        "- 格斗帮助：显示本帮助信息"
    )
    await luo9.send_group_message(group_id, help_text)

# 命令处理器
command_handlers = {
    r'^对战\s+(.+)$': battle.start_battle,
    r'^挑战\s+(.+)$': battle.start_battle,
    r'^格斗\s+(.+)$': battle.start_battle,
    r'^排行榜$': ranking.show_ranking,
    r'^我的背包$': inventory.show_inventory,
    r'^装备\s+(.+)$': inventory.equip_item,
    r'^卸下\s+(.+)$': inventory.unequip_item,
    r'^强化\s+(.+)$': enhancement.enhance_item,
    r'^商店$': item_manager.show_shop,
    r'^购买\s+(.+)$': item_manager.buy_item,
    r'^我的状态$': battle.show_status,
    r'^格斗帮助$': show_help
}

async def group_handle(message: GroupMessage):
    """处理群消息"""
    group_id = message.group_id
    user_id = message.user_id
    content = message.content
    
    # 确保数据目录存在
    os.makedirs(f"{value.data_path}/plugins/{config['name']}", exist_ok=True)
    
    # 初始化用户数据
    if not db.user_exists(user_id):
        db.create_user(user_id)
    
    # 处理命令
    for pattern, handler in command_handlers.items():
        match = re.match(pattern, content)
        if match and battle_limit.check(3):  # 限制3秒内只能执行一次命令
            battle_limit.handle()
            await handler(group_id, user_id, *match.groups())
            return
