from luo9.api_manager import luo9

class ItemManager:
    """物品管理系统，处理商店和物品购买"""
    
    def __init__(self, db):
        """初始化物品管理系统"""
        self.db = db
    
    async def show_shop(self, group_id, user_id, *args):
        """显示商店物品"""
        # 获取用户信息
        user = self.db.get_user(user_id)
        if not user:
            await luo9.send_group_message(group_id, "无法获取你的信息")
            return
        
        # 获取商店物品
        shop_items = self.db.get_shop_items()
        
        if not shop_items:
            await luo9.send_group_message(group_id, "商店暂无物品")
            return
        
        # 按类型分类物品
        weapons = [item for item in shop_items if item['type'] == 'weapon']
        armors = [item for item in shop_items if item['type'] == 'armor']
        accessories = [item for item in shop_items if item['type'] == 'accessory']
        consumables = [item for item in shop_items if item['type'] == 'consumable']
        
        # 构建商店消息
        shop_message = f"【商店】(你的金币: {user['gold']})\n"
        
        # 显示武器
        if weapons:
            shop_message += "\n武器:\n"
            for weapon in weapons[:5]:  # 只显示前5个
                shop_message += f"{weapon['name']} - {weapon['price']}金币\n"
                shop_message += f"  攻击+{weapon['attack']} 防御+{weapon['defense']} 生命+{weapon['hp']} 速度+{weapon['speed']}\n"
                shop_message += f"  {weapon['description']}\n"
        
        # 显示防具
        if armors:
            shop_message += "\n防具:\n"
            for armor in armors[:5]:
                shop_message += f"{armor['name']} - {armor['price']}金币\n"
                shop_message += f"  攻击+{armor['attack']} 防御+{armor['defense']} 生命+{armor['hp']} 速度+{armor['speed']}\n"
                shop_message += f"  {armor['description']}\n"
        
        # 显示饰品
        if accessories:
            shop_message += "\n饰品:\n"
            for accessory in accessories[:5]:
                shop_message += f"{accessory['name']} - {accessory['price']}金币\n"
                shop_message += f"  攻击+{accessory['attack']} 防御+{accessory['defense']} 生命+{accessory['hp']} 速度+{accessory['speed']}\n"
                shop_message += f"  {accessory['description']}\n"
        
        # 显示消耗品
        if consumables:
            shop_message += "\n消耗品:\n"
            for consumable in consumables[:5]:
                shop_message += f"{consumable['name']} - {consumable['price']}金币\n"
                shop_message += f"  {consumable['description']}\n"
        
        # 显示使用说明
        shop_message += "\n使用方法: 购买 [物品名]"
        
        await luo9.send_group_message(group_id, shop_message)
    
    async def buy_item(self, group_id, user_id, item_name):
        """购买物品"""
        # 购买物品
        success, message = self.db.buy_item(user_id, item_name)
        
        await luo9.send_group_message(group_id, message)
        
        # 如果购买成功，显示物品信息
        if success:
            item = self.db.get_user_item_by_name(user_id, item_name)
            if item:
                item_info = f"获得物品: {item['name']}\n"
                if item['attack'] > 0:
                    item_info += f"攻击+{item['attack']} "
                if item['defense'] > 0:
                    item_info += f"防御+{item['defense']} "
                if item['hp'] > 0:
                    item_info += f"生命+{item['hp']} "
                if item['speed'] > 0:
                    item_info += f"速度+{item['speed']}"
                
                await luo9.send_group_message(group_id, item_info)