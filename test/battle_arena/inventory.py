from luo9.api_manager import luo9

class Inventory:
    """背包系统，管理玩家物品"""
    
    def __init__(self, db, item_manager):
        """初始化背包系统"""
        self.db = db
        self.item_manager = item_manager
    
    async def show_inventory(self, group_id, user_id, *args):
        """显示用户背包"""
        items = self.db.get_user_items(user_id)
        
        if not items:
            await luo9.send_group_message(group_id, "你的背包是空的")
            return
        
        # 按类型分类物品
        weapons = [item for item in items if item['type'] == 'weapon']
        armors = [item for item in items if item['type'] == 'armor']
        accessories = [item for item in items if item['type'] == 'accessory']
        consumables = [item for item in items if item['type'] == 'consumable']
        
        # 构建背包消息
        inventory_message = f"【你的背包】\n"
        
        # 显示武器
        if weapons:
            inventory_message += "\n武器:\n"
            for weapon in weapons:
                equipped = "【已装备】" if weapon['is_equipped'] else ""
                inventory_message += f"{weapon['name']} (Lv.{weapon['level']}) {equipped}\n"
                inventory_message += f"  攻击+{weapon['attack']} 防御+{weapon['defense']} 生命+{weapon['hp']} 速度+{weapon['speed']}\n"
        
        # 显示防具
        if armors:
            inventory_message += "\n防具:\n"
            for armor in armors:
                equipped = "【已装备】" if armor['is_equipped'] else ""
                inventory_message += f"{armor['name']} (Lv.{armor['level']}) {equipped}\n"
                inventory_message += f"  攻击+{armor['attack']} 防御+{armor['defense']} 生命+{armor['hp']} 速度+{armor['speed']}\n"
        
        # 显示饰品
        if accessories:
            inventory_message += "\n饰品:\n"
            for accessory in accessories:
                equipped = "【已装备】" if accessory['is_equipped'] else ""
                inventory_message += f"{accessory['name']} (Lv.{accessory['level']}) {equipped}\n"
                inventory_message += f"  攻击+{accessory['attack']} 防御+{accessory['defense']} 生命+{accessory['hp']} 速度+{accessory['speed']}\n"
        
        # 显示消耗品
        if consumables:
            inventory_message += "\n消耗品:\n"
            for consumable in consumables:
                inventory_message += f"{consumable['name']}\n"
        
        # 显示使用说明
        inventory_message += "\n使用方法: 装备 [物品名] / 卸下 [物品名] / 强化 [物品名]"
        
        await luo9.send_group_message(group_id, inventory_message)
    
    async def equip_item(self, group_id, user_id, item_name):
        """装备物品"""
        # 获取物品信息
        item = self.db.get_user_item_by_name(user_id, item_name)
        
        if not item:
            await luo9.send_group_message(group_id, f"你没有名为 {item_name} 的物品")
            return
        
        # 检查物品是否可装备
        if item['type'] == 'consumable':
            await luo9.send_group_message(group_id, f"{item_name} 是消耗品，无法装备")
            return
        
        # 检查物品是否已装备
        if item['is_equipped']:
            await luo9.send_group_message(group_id, f"{item_name} 已经装备了")
            return
        
        # 装备物品
        success = self.db.equip_item(user_id, item['item_id'])
        
        if success:
            await luo9.send_group_message(group_id, f"成功装备 {item_name}")
        else:
            await luo9.send_group_message(group_id, f"装备 {item_name} 失败")
    
    async def unequip_item(self, group_id, user_id, item_name):
        """卸下物品"""
        # 获取物品信息
        item = self.db.get_user_item_by_name(user_id, item_name)
        
        if not item:
            await luo9.send_group_message(group_id, f"你没有名为 {item_name} 的物品")
            return
        
        # 检查物品是否已装备
        if not item['is_equipped']:
            await luo9.send_group_message(group_id, f"{item_name} 尚未装备")
            return
        
        # 卸下物品
        success = self.db.unequip_item(user_id, item['item_id'])
        
        if success:
            await luo9.send_group_message(group_id, f"成功卸下 {item_name}")
        else:
            await luo9.send_group_message(group_id, f"卸下 {item_name} 失败")