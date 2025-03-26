import os
import json
import sqlite3
from config import get_value

value = get_value()

class Database:
    """数据库管理类，负责存储和读取游戏数据"""
    
    def __init__(self):
        """初始化数据库连接"""
        self.db_path = f"{value.data_path}/plugins/battle_arena/battle_arena.db"
        self._init_db()
    
    def _init_db(self):
        """初始化数据库表结构"""
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # 创建用户表
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS users (
            user_id INTEGER PRIMARY KEY,
            name TEXT,
            level INTEGER DEFAULT 1,
            exp INTEGER DEFAULT 0,
            gold INTEGER DEFAULT 1000,
            hp INTEGER DEFAULT 100,
            attack INTEGER DEFAULT 10,
            defense INTEGER DEFAULT 5,
            speed INTEGER DEFAULT 5,
            wins INTEGER DEFAULT 0,
            losses INTEGER DEFAULT 0,
            equipped_weapon INTEGER DEFAULT NULL,
            equipped_armor INTEGER DEFAULT NULL,
            equipped_accessory INTEGER DEFAULT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        ''')
        
        # 创建物品表
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS items (
            item_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            name TEXT,
            type TEXT,
            rarity TEXT,
            level INTEGER DEFAULT 1,
            attack INTEGER DEFAULT 0,
            defense INTEGER DEFAULT 0,
            hp INTEGER DEFAULT 0,
            speed INTEGER DEFAULT 0,
            special_effect TEXT,
            is_equipped INTEGER DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(user_id)
        )
        ''')
        
        # 创建商店物品表
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS shop_items (
            item_id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            type TEXT,
            rarity TEXT,
            attack INTEGER DEFAULT 0,
            defense INTEGER DEFAULT 0,
            hp INTEGER DEFAULT 0,
            speed INTEGER DEFAULT 0,
            special_effect TEXT,
            price INTEGER,
            description TEXT
        )
        ''')
        
        # 创建战斗记录表
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS battle_records (
            record_id INTEGER PRIMARY KEY AUTOINCREMENT,
            attacker_id INTEGER,
            defender_id INTEGER,
            winner_id INTEGER,
            battle_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            battle_log TEXT,
            FOREIGN KEY (attacker_id) REFERENCES users(user_id),
            FOREIGN KEY (defender_id) REFERENCES users(user_id),
            FOREIGN KEY (winner_id) REFERENCES users(user_id)
        )
        ''')
        
        # 初始化商店物品
        self._init_shop_items(cursor)
        
        conn.commit()
        conn.close()
    
    def _init_shop_items(self, cursor):
        """初始化商店物品"""
        # 检查商店是否已有物品
        cursor.execute("SELECT COUNT(*) FROM shop_items")
        if cursor.fetchone()[0] > 0:
            return
        
        # 武器
        weapons = [
            ("新手木剑", "weapon", "common", 5, 0, 0, 0, "", 100, "新手的第一把武器"),
            ("铁剑", "weapon", "common", 10, 0, 0, 0, "", 300, "普通的铁剑，略有锋利"),
            ("精钢长剑", "weapon", "uncommon", 20, 0, 0, 2, "", 800, "精钢打造的长剑，附带速度加成"),
            ("火焰魔剑", "weapon", "rare", 35, 0, 0, 0, "fire_damage", 2000, "附带火焰伤害的魔剑"),
            ("霜之哀伤", "weapon", "epic", 50, 10, 0, 0, "ice_slow", 5000, "传说中的霜之哀伤，可减缓敌人速度"),
            ("屠龙宝刀", "weapon", "legendary", 80, 0, 20, 0, "dragon_slayer", 10000, "号称屠龙的宝刀，附带生命值加成")
        ]
        
        # 防具
        armors = [
            ("皮革护甲", "armor", "common", 0, 5, 10, 0, "", 100, "简单的皮革护甲"),
            ("铁甲", "armor", "common", 0, 15, 0, -1, "", 300, "沉重的铁甲，会减少速度"),
            ("精钢护甲", "armor", "uncommon", 0, 25, 0, 0, "", 800, "精钢打造的护甲"),
            ("龙鳞甲", "armor", "rare", 5, 40, 0, 0, "fire_resist", 2000, "龙鳞制成的护甲，附带攻击加成"),
            ("圣骑士铠甲", "armor", "epic", 10, 60, 30, 0, "holy_protection", 5000, "圣骑士的铠甲，附带生命值加成"),
            ("神圣守护", "armor", "legendary", 20, 80, 50, 0, "divine_shield", 10000, "传说中的神圣守护，全属性加成")
        ]
        
        # 饰品
        accessories = [
            ("幸运符", "accessory", "common", 0, 0, 5, 5, "luck", 100, "带来一点点幸运"),
            ("力量戒指", "accessory", "common", 5, 0, 0, 0, "", 300, "增加攻击力的戒指"),
            ("守护项链", "accessory", "uncommon", 0, 5, 10, 0, "", 800, "增加防御和生命的项链"),
            ("迅捷之靴", "accessory", "rare", 0, 0, 0, 15, "dodge", 2000, "大幅提高速度，有几率闪避攻击"),
            ("贤者宝石", "accessory", "epic", 15, 15, 0, 0, "mana_regen", 5000, "攻防双加的宝石"),
            ("时间沙漏", "accessory", "legendary", 10, 10, 20, 10, "time_control", 10000, "传说中的时间沙漏，全属性加成")
        ]
        
        # 消耗品
        consumables = [
            ("小型生命药水", "consumable", "common", 0, 0, 20, 0, "instant_heal", 50, "恢复少量生命值"),
            ("中型生命药水", "consumable", "uncommon", 0, 0, 50, 0, "instant_heal", 150, "恢复中等生命值"),
            ("大型生命药水", "consumable", "rare", 0, 0, 100, 0, "instant_heal", 400, "恢复大量生命值"),
            ("力量药剂", "consumable", "uncommon", 10, 0, 0, 0, "temp_buff", 200, "暂时提高攻击力"),
            ("防御药剂", "consumable", "uncommon", 0, 10, 0, 0, "temp_buff", 200, "暂时提高防御力"),
            ("速度药剂", "consumable", "uncommon", 0, 0, 0, 10, "temp_buff", 200, "暂时提高速度")
        ]
        
        # 插入所有物品
        all_items = weapons + armors + accessories + consumables
        for item in all_items:
            cursor.execute('''
            INSERT INTO shop_items (name, type, rarity, attack, defense, hp, speed, special_effect, price, description)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', item)
    
    def user_exists(self, user_id):
        """检查用户是否存在"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM users WHERE user_id = ?", (user_id,))
        exists = cursor.fetchone()[0] > 0
        conn.close()
        return exists
    
    def create_user(self, user_id, name=None):
        """创建新用户"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # 如果没有提供名称，使用用户ID作为名称
        if name is None:
            name = f"玩家{user_id}"
            
        cursor.execute('''
        INSERT INTO users (user_id, name) VALUES (?, ?)
        ''', (user_id, name))
        
        # 给新用户一把初始武器
        cursor.execute('''
        INSERT INTO items (user_id, name, type, rarity, attack, defense, hp, speed, special_effect)
        VALUES (?, '新手木剑', 'weapon', 'common', 5, 0, 0, 0, '')
        ''', (user_id,))
        
        conn.commit()
        conn.close()
        return True
    
    def get_user(self, user_id):
        """获取用户信息"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row  # 使结果可以通过列名访问
        cursor = conn.cursor()
        
        cursor.execute('''
        SELECT * FROM users WHERE user_id = ?
        ''', (user_id,))
        
        user = cursor.fetchone()
        conn.close()
        
        if user:
            return dict(user)
        return None
    
    def update_user(self, user_id, **kwargs):
        """更新用户信息"""
        if not kwargs:
            return False
            
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # 构建更新语句
        set_clause = ", ".join([f"{key} = ?" for key in kwargs.keys()])
        values = list(kwargs.values())
        values.append(user_id)
        
        cursor.execute(f'''
        UPDATE users SET {set_clause} WHERE user_id = ?
        ''', values)
        
        conn.commit()
        conn.close()
        return cursor.rowcount > 0
    
    def add_exp(self, user_id, exp_amount):
        """增加用户经验值，并处理升级"""
        user = self.get_user(user_id)
        if not user:
            return False
            
        # 计算新的经验值
        new_exp = user['exp'] + exp_amount
        new_level = user['level']
        
        # 检查是否升级 (简单公式: 下一级所需经验 = 当前等级 * 100)
        while new_exp >= new_level * 100:
            new_exp -= new_level * 100
            new_level += 1
        
        # 更新用户数据
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
        UPDATE users SET exp = ?, level = ?, 
        hp = hp + ?, attack = attack + ?, defense = defense + ?, speed = speed + ?
        WHERE user_id = ?
        ''', (new_exp, new_level, 
              (new_level - user['level']) * 10,  # 每升一级增加10点生命
              (new_level - user['level']) * 2,   # 每升一级增加2点攻击
              (new_level - user['level']) * 1,   # 每升一级增加1点防御
              (new_level - user['level']) * 1,   # 每升一级增加1点速度
              user_id))
        
        conn.commit()
        conn.close()
        
        return new_level > user['level'], new_level
    
    def add_gold(self, user_id, gold_amount):
        """增加或减少用户金币"""
        user = self.get_user(user_id)
        if not user:
            return False
            
        new_gold = max(0, user['gold'] + gold_amount)  # 确保金币不会为负
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
        UPDATE users SET gold = ? WHERE user_id = ?
        ''', (new_gold, user_id))
        
        conn.commit()
        conn.close()
        return True
    
    # 物品相关方法
    def get_item(self, item_id):
        """获取物品信息"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute('''
        SELECT * FROM items WHERE item_id = ?
        ''', (item_id,))
        
        item = cursor.fetchone()
        conn.close()
        
        if item:
            return dict(item)
        return None
    
    def get_user_items(self, user_id):
        """获取用户所有物品"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute('''
        SELECT * FROM items WHERE user_id = ?
        ''', (user_id,))
        
        items = [dict(item) for item in cursor.fetchall()]
        conn.close()
        
        return items
    
    def get_equipped_items(self, user_id):
        """获取用户已装备的物品"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute('''
        SELECT * FROM items WHERE user_id = ? AND is_equipped = 1
        ''', (user_id,))
        
        items = [dict(item) for item in cursor.fetchall()]
        conn.close()
        
        return items
    
    def add_item(self, user_id, item_name):
        """给用户添加物品（从商店购买）"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        # 获取商店物品信息
        cursor.execute('''
        SELECT * FROM shop_items WHERE name = ?
        ''', (item_name,))
        
        shop_item = cursor.fetchone()
        if not shop_item:
            conn.close()
            return False
            
        shop_item = dict(shop_item)
        
        # 添加物品到用户背包
        cursor.execute('''
        INSERT INTO items (user_id, name, type, rarity, attack, defense, hp, speed, special_effect)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (user_id, shop_item['name'], shop_item['type'], shop_item['rarity'],
              shop_item['attack'], shop_item['defense'], shop_item['hp'], 
              shop_item['speed'], shop_item['special_effect']))
        
        conn.commit()
        conn.close()
        return True
    
    def equip_item(self, user_id, item_id):
        """装备物品"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        # 获取物品信息
        cursor.execute('''
        SELECT * FROM items WHERE item_id = ? AND user_id = ?
        ''', (item_id, user_id))
        
        item = cursor.fetchone()
        if not item:
            conn.close()
            return False
            
        item = dict(item)
        
        # 获取用户信息
        cursor.execute('''
        SELECT * FROM users WHERE user_id = ?
        ''', (user_id,))
        
        user = dict(cursor.fetchone())
        
        # 根据物品类型，先卸下同类型的已装备物品
        if item['type'] == 'weapon':
            cursor.execute('''
            UPDATE items SET is_equipped = 0 
            WHERE user_id = ? AND type = 'weapon' AND is_equipped = 1
            ''', (user_id,))
            
            # 更新用户装备信息
            cursor.execute('''
            UPDATE users SET equipped_weapon = ? WHERE user_id = ?
            ''', (item_id, user_id))
            
        elif item['type'] == 'armor':
            cursor.execute('''
            UPDATE items SET is_equipped = 0 
            WHERE user_id = ? AND type = 'armor' AND is_equipped = 1
            ''', (user_id,))
            
            # 更新用户装备信息
            cursor.execute('''
            UPDATE users SET equipped_armor = ? WHERE user_id = ?
            ''', (item_id, user_id))
            
        elif item['type'] == 'accessory':
            cursor.execute('''
            UPDATE items SET is_equipped = 0 
            WHERE user_id = ? AND type = 'accessory' AND is_equipped = 1
            ''', (user_id,))
            
            # 更新用户装备信息
            cursor.execute('''
            UPDATE users SET equipped_accessory = ? WHERE user_id = ?
            ''', (item_id, user_id))
        
        # 装备新物品
        cursor.execute('''
        UPDATE items SET is_equipped = 1 WHERE item_id = ?
        ''', (item_id,))
        
        conn.commit()
        conn.close()
        return True
    
    def unequip_item(self, user_id, item_id):
        """卸下物品"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        # 获取物品信息
        cursor.execute('''
        SELECT * FROM items WHERE item_id = ? AND user_id = ?
        ''', (item_id, user_id))
        
        item = cursor.fetchone()
        if not item or item['is_equipped'] == 0:
            conn.close()
            return False
            
        item = dict(item)
        
        # 卸下物品
        cursor.execute('''
        UPDATE items SET is_equipped = 0 WHERE item_id = ?
        ''', (item_id,))
        
        # 更新用户装备信息
        if item['type'] == 'weapon':
            cursor.execute('''
            UPDATE users SET equipped_weapon = NULL WHERE user_id = ?
            ''', (user_id,))
        elif item['type'] == 'armor':
            cursor.execute('''
            UPDATE users SET equipped_armor = NULL WHERE user_id = ?
            ''', (user_id,))
        elif item['type'] == 'accessory':
            cursor.execute('''
            UPDATE users SET equipped_accessory = NULL WHERE user_id = ?
            ''', (user_id,))
        
        conn.commit()
        conn.close()
        return True
    
    def enhance_item(self, item_id, success_rate=0.7):
        """强化物品"""
        import random
        
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        # 获取物品信息
        cursor.execute('''
        SELECT * FROM items WHERE item_id = ?
        ''', (item_id,))
        
        item = cursor.fetchone()
        if not item:
            conn.close()
            return False, None
            
        item = dict(item)
        
        # 判断强化是否成功
        success = random.random() < success_rate
        
        if success:
            # 强化成功，提升物品等级和属性
            new_level = item['level'] + 1
            
            # 根据物品稀有度和类型增加不同的属性值
            rarity_multiplier = {
                'common': 1,
                'uncommon': 1.2,
                'rare': 1.5,
                'epic': 2,
                'legendary': 3
            }.get(item['rarity'], 1)
            
            # 计算属性增加值
            attack_increase = 0
            defense_increase = 0
            hp_increase = 0
            speed_increase = 0
            
            if item['type'] == 'weapon':
                attack_increase = int(2 * rarity_multiplier)
            elif item['type'] == 'armor':
                defense_increase = int(2 * rarity_multiplier)
                hp_increase = int(5 * rarity_multiplier)
            elif item['type'] == 'accessory':
                attack_increase = int(1 * rarity_multiplier)
                defense_increase = int(1 * rarity_multiplier)
                speed_increase = int(1 * rarity_multiplier)
            
            # 更新物品属性
            cursor.execute('''
            UPDATE items SET 
            level = ?,
            attack = attack + ?,
            defense = defense + ?,
            hp = hp + ?,
            speed = speed + ?
            WHERE item_id = ?
            ''', (new_level, attack_increase, defense_increase, hp_increase, speed_increase, item_id))
            
            conn.commit()
            
            # 获取更新后的物品信息
            cursor.execute('''
            SELECT * FROM items WHERE item_id = ?
            ''', (item_id,))
            
            updated_item = dict(cursor.fetchone())
            conn.close()
            
            return True, updated_item
        else:
            # 强化失败
            conn.close()
            return False, item
    
    # 战斗相关方法
    def record_battle(self, attacker_id, defender_id, winner_id, battle_log):
        """记录战斗结果"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
        INSERT INTO battle_records (attacker_id, defender_id, winner_id, battle_log)
        VALUES (?, ?, ?, ?)
        ''', (attacker_id, defender_id, winner_id, json.dumps(battle_log)))
        
        # 更新胜负记录
        if winner_id == attacker_id:
            cursor.execute('''
            UPDATE users SET wins = wins + 1 WHERE user_id = ?
            ''', (attacker_id,))
            cursor.execute('''
            UPDATE users SET losses = losses + 1 WHERE user_id = ?
            ''', (defender_id,))
        else:
            cursor.execute('''
            UPDATE users SET losses = losses + 1 WHERE user_id = ?
            ''', (attacker_id,))
            cursor.execute('''
            UPDATE users SET wins = wins + 1 WHERE user_id = ?
            ''', (defender_id,))
        
        conn.commit()
        conn.close()
        return True
    
    def get_battle_records(self, user_id, limit=5):
        """获取用户最近的战斗记录"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute('''
        SELECT * FROM battle_records 
        WHERE attacker_id = ? OR defender_id = ?
        ORDER BY battle_time DESC
        LIMIT ?
        ''', (user_id, user_id, limit))
        
        records = [dict(record) for record in cursor.fetchall()]
        conn.close()
        
        return records
    
    def get_ranking(self, limit=10):
        """获取战力排行榜"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        # 计算战力：攻击 + 防御 + 生命/10 + 速度 + (胜场数 - 败场数) * 5
        cursor.execute('''
        SELECT *, (attack + defense + hp/10 + speed + (wins - losses) * 5) as power
        FROM users
        ORDER BY power DESC
        LIMIT ?
        ''', (limit,))
        
        ranking = [dict(user) for user in cursor.fetchall()]
        conn.close()
        
        return ranking
    
    def get_shop_items(self, item_type=None, limit=10):
        """获取商店物品列表"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        if item_type:
            cursor.execute('''
            SELECT * FROM shop_items
            WHERE type = ?
            ORDER BY price ASC
            LIMIT ?
            ''', (item_type, limit))
        else:
            cursor.execute('''
            SELECT * FROM shop_items
            ORDER BY price ASC
            LIMIT ?
            ''', (limit,))
        
        items = [dict(item) for item in cursor.fetchall()]
        conn.close()
        
        return items
    
    def get_shop_item_by_name(self, item_name):
        """根据名称获取商店物品"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute('''
        SELECT * FROM shop_items
        WHERE name = ?
        ''', (item_name,))
        
        item = cursor.fetchone()
        conn.close()
        
        if item:
            return dict(item)
        return None
    
    def buy_item(self, user_id, item_name):
        """用户购买物品"""
        # 获取用户信息
        user = self.get_user(user_id)
        if not user:
            return False, "用户不存在"
        
        # 获取商店物品信息
        shop_item = self.get_shop_item_by_name(item_name)
        if not shop_item:
            return False, "商店中没有该物品"
        
        # 检查用户金币是否足够
        if user['gold'] < shop_item['price']:
            return False, "金币不足"
        
        # 扣除金币
        self.add_gold(user_id, -shop_item['price'])
        
        # 添加物品到用户背包
        self.add_item(user_id, item_name)
        
        return True, f"成功购买 {item_name}"
    
    def get_user_item_by_name(self, user_id, item_name):
        """根据名称获取用户物品"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        cursor.execute('''
        SELECT * FROM items
        WHERE user_id = ? AND name = ?
        ''', (user_id, item_name))
        
        item = cursor.fetchone()
        conn.close()
        
        if item:
            return dict(item)
        return None
    
    def calculate_user_power(self, user_id):
        """计算用户战力"""
        user = self.get_user(user_id)
        if not user:
            return 0
        
        # 基础战力
        base_power = user['attack'] + user['defense'] + user['hp']/10 + user['speed']
        
        # 装备加成
        equipped_items = self.get_equipped_items(user_id)
        equipment_power = sum(item['attack'] + item['defense'] + item['hp']/10 + item['speed'] 
                             for item in equipped_items)
        
        # 胜负记录加成
        record_power = (user['wins'] - user['losses']) * 5
        
        # 等级加成
        level_power = user['level'] * 10
        
        total_power = base_power + equipment_power + record_power + level_power
        return round(total_power)
    
    def get_user_battle_stats(self, user_id):
        """获取用户战斗统计信息"""
        user = self.get_user(user_id)
        if not user:
            return None
        
        # 获取装备加成
        equipped_items = self.get_equipped_items(user_id)
        
        # 计算总属性
        total_attack = user['attack']
        total_defense = user['defense']
        total_hp = user['hp']
        total_speed = user['speed']
        
        for item in equipped_items:
            total_attack += item['attack']
            total_defense += item['defense']
            total_hp += item['hp']
            total_speed += item['speed']
        
        # 计算战力
        power = self.calculate_user_power(user_id)
        
        return {
            'user_id': user_id,
            'name': user['name'],
            'level': user['level'],
            'exp': user['exp'],
            'gold': user['gold'],
            'attack': total_attack,
            'defense': total_defense,
            'hp': total_hp,
            'speed': total_speed,
            'wins': user['wins'],
            'losses': user['losses'],
            'power': power,
            'equipped_items': equipped_items
        }
    
    def delete_item(self, item_id):
        """删除物品"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
        DELETE FROM items WHERE item_id = ?
        ''', (item_id,))
        
        success = cursor.rowcount > 0
        conn.commit()
        conn.close()
        
        return success
    
    def get_daily_reward(self, user_id):
        """获取每日奖励"""
        import time
        from datetime import datetime, timedelta
        
        user = self.get_user(user_id)
        if not user:
            return False, "用户不存在"
        
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        # 检查是否已经领取过今日奖励
        cursor.execute('''
        SELECT * FROM daily_rewards
        WHERE user_id = ? AND date(reward_time) = date('now')
        ''', (user_id,))
        
        if cursor.fetchone():
            conn.close()
            return False, "今天已经领取过奖励了"
        
        # 计算奖励金额 (基础100金币 + 等级加成)
        reward_gold = 100 + user['level'] * 10
        
        # 更新用户金币
        self.add_gold(user_id, reward_gold)
        
        # 记录领奖时间
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS daily_rewards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            reward_gold INTEGER,
            reward_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(user_id)
        )
        ''')
        
        cursor.execute('''
        INSERT INTO daily_rewards (user_id, reward_gold)
        VALUES (?, ?)
        ''', (user_id, reward_gold))
        
        conn.commit()
        conn.close()
        
        return True, f"成功领取每日奖励: {reward_gold} 金币"