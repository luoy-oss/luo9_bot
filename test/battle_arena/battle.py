import random
import json
import re
import os
import time
from luo9.api_manager import luo9
from config import get_value
from .gif_generator import GifGenerator

value = get_value()

class Battle:
    """战斗系统类，处理玩家之间的对战"""
    
    def __init__(self, db):
        """初始化战斗系统"""
        self.db = db
        self.gif_generator = GifGenerator()
        self.ongoing_battles = {}  # 记录正在进行的战斗
    
    async def start_battle(self, group_id, user_id, target_name):
        """开始一场战斗"""
        # 查找目标用户
        target_id = None
        # 从@消息中提取用户ID
        match = re.search(r'\[CQ:at,qq=(\d+)\]', target_name)
        if match:
            target_id = int(match.group(1))
        
        if not target_id:
            await luo9.send_group_message(group_id, "无法识别目标用户，请确保正确@了对方")
            return
        
        # 不能和自己战斗
        if target_id == user_id:
            await luo9.send_group_message(group_id, "不能和自己战斗！")
            return
        
        # 确保目标用户存在
        if not self.db.user_exists(target_id):
            self.db.create_user(target_id)
        
        # 获取双方战斗数据
        attacker_stats = self.db.get_user_battle_stats(user_id)
        defender_stats = self.db.get_user_battle_stats(target_id)
        
        if not attacker_stats or not defender_stats:
            await luo9.send_group_message(group_id, "获取战斗数据失败")
            return
        
        # 初始化战斗日志
        battle_log = []
        
        # 初始化战斗状态
        attacker_hp = attacker_stats['hp']
        defender_hp = defender_stats['hp']
        
        # 确保初始化winner变量
        winner = None
        
        # 战斗循环
        round_num = 1
        while attacker_hp > 0 and defender_hp > 0 and round_num <= 10:  # 最多10回合
            battle_log.append(f"第{round_num}回合:")
            # 根据速度决定谁先攻击
            if attacker_stats['speed'] >= defender_stats['speed']:
                # 攻击者先攻
                damage = max(1, attacker_stats['attack'] - defender_stats['defense'] // 2)
                defender_hp -= damage
                battle_log.append(f"\n{attacker_stats['name']}对{defender_stats['name']}造成{damage}点伤害")
                
                # 检查是否结束战斗
                if defender_hp <= 0:
                    winner = user_id
                    battle_log.append(f"\n{defender_stats['name']}倒下了，{attacker_stats['name']}获胜！")
                    break
                
                # 防御者反击
                damage = max(1, defender_stats['attack'] - attacker_stats['defense'] // 2)
                attacker_hp -= damage
                battle_log.append(f"\n{defender_stats['name']}对{attacker_stats['name']}造成{damage}点伤害")
                
                # 检查是否结束战斗
                if attacker_hp <= 0:
                    winner = target_id
                    battle_log.append(f"\n{attacker_stats['name']}倒下了，{defender_stats['name']}获胜！")
                    break
            else:
                # 防御者先攻
                damage = max(1, defender_stats['attack'] - attacker_stats['defense'] // 2)
                attacker_hp -= damage
                battle_log.append(f"\n{defender_stats['name']}对{attacker_stats['name']}造成{damage}点伤害")
                
                # 检查是否结束战斗
                if attacker_hp <= 0:
                    winner = target_id
                    battle_log.append(f"\n{attacker_stats['name']}倒下了，{defender_stats['name']}获胜！")
                    break
                
                # 攻击者反击
                damage = max(1, attacker_stats['attack'] - defender_stats['defense'] // 2)
                defender_hp -= damage
                battle_log.append(f"\n{attacker_stats['name']}对{defender_stats['name']}造成{damage}点伤害")
                
                # 检查是否结束战斗
                if defender_hp <= 0:
                    winner = user_id
                    battle_log.append(f"\n{defender_stats['name']}倒下了，{attacker_stats['name']}获胜！")
                    break
            
            round_num += 1
        
        # 如果达到最大回合数仍未分出胜负，则根据剩余血量决定胜者
        if winner is None:
            if attacker_hp > defender_hp:
                winner = user_id
                battle_log.append(f"\n战斗结束，{attacker_stats['name']}以更多的生命值获胜！")
            elif defender_hp > attacker_hp:
                winner = target_id
                battle_log.append(f"\n战斗结束，{defender_stats['name']}以更多的生命值获胜！")
            else:
                # 平局情况，随机决定胜者
                winner = random.choice([user_id, target_id])
                battle_log.append(f"\n战斗陷入僵局，经过抽签，{attacker_stats['name'] if winner == user_id else defender_stats['name']}获胜！")
        
        # 记录战斗结果
        self.db.record_battle(user_id, target_id, winner, battle_log)
        
        # 奖励胜利者
        exp_reward = 20
        gold_reward = 50
        
        if winner == user_id:
            self.db.add_exp(user_id, exp_reward)
            self.db.add_gold(user_id, gold_reward)
            battle_log.append(f"\n{attacker_stats['name']}获得{exp_reward}经验和{gold_reward}金币")
        else:
            self.db.add_exp(target_id, exp_reward)
            self.db.add_gold(target_id, gold_reward)
            battle_log.append(f"\n{defender_stats['name']}获得{exp_reward}经验和{gold_reward}金币")
        
        # 生成战斗GIF
        try:
            # 准备战斗数据用于GIF生成
            attacker_data = {
                'name': attacker_stats['name'],
                'hp': attacker_stats['hp'],
                'hp_history': [attacker_stats['hp']]
            }
            
            defender_data = {
                'name': defender_stats['name'],
                'hp': defender_stats['hp'],
                'hp_history': [defender_stats['hp']]
            }
            
            # 构建战斗日志
            battle_log_for_gif = []
            for log in battle_log:
                if "造成" in log and "点伤害" in log:
                    log_modified = log.replace("造成", "造成了")
                    battle_log_for_gif.append(log_modified)
                elif "倒下了" in log:
                    if attacker_stats['name'] in log and "倒下了":
                        battle_log_for_gif.append(f"{attacker_stats['name']}被击败了")
                    elif defender_stats['name'] in log and "倒下了":
                        battle_log_for_gif.append(f"{defender_stats['name']}被击败了")
            
            # 生成GIF文件
            timestamp = int(time.time())
            gif_path = f"{value.data_path}/plugins/battle_arena/battles/{user_id}_vs_{target_id}_{timestamp}.gif"
            os.makedirs(os.path.dirname(gif_path), exist_ok=True)
            
            # 调用GIF生成器
            output_file = await self.gif_generator.generate_battle_gif(battle_log_for_gif, attacker_data, defender_data)
            
            # 发送战斗结果文本
            result_text = "\n".join(battle_log)
            await luo9.send_group_message(group_id, result_text)
            
            # 发送战斗GIF
            if output_file and os.path.exists(output_file):
                await luo9.send_group_image(group_id, file=output_file)
            else:
                await luo9.send_group_message(group_id, "[GIF生成失败: 无法找到生成的图片文件]")
            
        except Exception as e:
            # 如果GIF生成失败，只发送文本结果
            result_text = "\n".join(battle_log)
            await luo9.send_group_message(group_id, f"{result_text}\n\n[GIF生成失败: {str(e)}]")
    
    # 在 Battle 类的 show_status 方法中
    async def show_status(self, group_id, user_id, *args):
        """显示用户状态"""
        # 将 db.get_user_data(user_id) 改为 db.get_user_battle_stats(user_id)
        user_stats = self.db.get_user_battle_stats(user_id)
        
        if not user_stats:
            await luo9.send_group_message(group_id, "获取用户状态失败")
            return
        
        # 构建状态信息
        status_text = (
            f"【{user_stats['name']}的状态】\n"
            f"等级: {user_stats['level']} (经验: {user_stats['exp']})\n"
            f"金币: {user_stats['gold']}\n"
            f"攻击力: {user_stats['attack']}\n"
            f"防御力: {user_stats['defense']}\n"
            f"生命值: {user_stats['hp']}\n"
            f"速度: {user_stats['speed']}\n"
            f"战绩: {user_stats['wins']}胜 {user_stats['losses']}负\n"
            f"战力: {user_stats['power']}"
        )
        
        # 显示已装备物品
        if user_stats['equipped_items']:
            status_text += "\n\n【已装备物品】"
            for item in user_stats['equipped_items']:
                status_text += f"\n{item['name']} (Lv.{item['level']}): "
                stats = []
                if item['attack'] > 0:
                    stats.append(f"攻击+{item['attack']}")
                if item['defense'] > 0:
                    stats.append(f"防御+{item['defense']}")
                if item['hp'] > 0:
                    stats.append(f"生命+{item['hp']}")
                if item['speed'] > 0:
                    stats.append(f"速度+{item['speed']}")
                status_text += ", ".join(stats)
        
        await luo9.send_group_message(group_id, status_text)