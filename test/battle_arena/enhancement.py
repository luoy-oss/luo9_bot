import random
from luo9.api_manager import luo9

class Enhancement:
    """物品强化系统，提升物品属性"""
    
    def __init__(self, db, item_manager):
        """初始化强化系统"""
        self.db = db
        self.item_manager = item_manager
    
    async def enhance_item(self, group_id, user_id, item_name):
        """强化物品"""
        # 获取物品信息
        item = self.db.get_user_item_by_name(user_id, item_name)
        
        if not item:
            await luo9.send_group_message(group_id, f"你没有名为 {item_name} 的物品")
            return
        
        # 获取用户信息
        user = self.db.get_user(user_id)
        
        # 计算强化费用 (基础100金币 + 物品等级 * 50 + 稀有度加成)
        rarity_multiplier = {
            'common': 1,
            'uncommon': 1.5,
            'rare': 2,
            'epic': 3,
            'legendary': 5
        }.get(item['rarity'], 1)
        
        enhance_cost = int((100 + item['level'] * 50) * rarity_multiplier)
        
        # 检查金币是否足够
        if user['gold'] < enhance_cost:
            await luo9.send_group_message(group_id, f"强化 {item_name} 需要 {enhance_cost} 金币，但你只有 {user['gold']} 金币")
            return
        
        # 扣除金币
        self.db.add_gold(user_id, -enhance_cost)
        
        # 计算成功率 (基础70%，随等级降低)
        success_rate = max(0.3, 0.7 - (item['level'] - 1) * 0.05)
        
        # 尝试强化
        success, updated_item = self.db.enhance_item(item['item_id'], success_rate)
        
        if success:
            # 构建成功消息
            success_message = f"花费 {enhance_cost} 金币强化 {item_name} 成功！\n"
            success_message += f"{item_name} 升级到 Lv.{updated_item['level']}\n"
            
            # 计算属性提升
            attack_increase = updated_item['attack'] - item['attack']
            defense_increase = updated_item['defense'] - item['defense']
            hp_increase = updated_item['hp'] - item['hp']
            speed_increase = updated_item['speed'] - item['speed']
            
            if attack_increase > 0:
                success_message += f"攻击力 +{attack_increase} "
            if defense_increase > 0:
                success_message += f"防御力 +{defense_increase} "
            if hp_increase > 0:
                success_message += f"生命值 +{hp_increase} "
            if speed_increase > 0:
                success_message += f"速度 +{speed_increase}"
            
            await luo9.send_group_message(group_id, success_message)
        else:
            # 构建失败消息
            fail_message = f"花费 {enhance_cost} 金币强化 {item_name} 失败！\n"
            fail_message += "物品等级和属性保持不变。"
            
            await luo9.send_group_message(group_id, fail_message)