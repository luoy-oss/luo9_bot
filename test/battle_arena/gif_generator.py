import os
import random
from PIL import Image, ImageDraw, ImageFont
import imageio
from config import get_value

value = get_value()

class GifGenerator:
    """GIF生成器，用于创建战斗动画"""
    
    def __init__(self):
        """初始化GIF生成器"""
        self.assets_path = f"{value.data_path}/plugins/battle_arena/assets"
        self.output_path = f"{value.data_path}/plugins/battle_arena/output"
        
        # 确保目录存在
        os.makedirs(f"{self.assets_path}/characters", exist_ok=True)
        os.makedirs(f"{self.assets_path}/items", exist_ok=True)
        os.makedirs(f"{self.assets_path}/backgrounds", exist_ok=True)
        os.makedirs(self.output_path, exist_ok=True)
        
        # 创建默认资源
        self._create_default_assets()
    
    def _create_default_assets(self):
        """创建默认资源文件"""
        # 创建默认背景
        if not os.path.exists(f"{self.assets_path}/backgrounds/default.png"):
            bg = Image.new('RGB', (800, 600), (50, 50, 80))
            draw = ImageDraw.Draw(bg)
            
            # 绘制简单的背景
            for i in range(50):
                x1, y1 = random.randint(0, 800), random.randint(0, 600)
                x2, y2 = x1 + random.randint(1, 3), y1 + random.randint(1, 3)
                draw.rectangle([x1, y1, x2, y2], fill=(70, 70, 100))
            
            bg.save(f"{self.assets_path}/backgrounds/default.png")
        
        # 创建默认角色
        if not os.path.exists(f"{self.assets_path}/characters/default.png"):
            char = Image.new('RGBA', (100, 200), (0, 0, 0, 0))
            draw = ImageDraw.Draw(char)
            
            # 绘制简单的角色轮廓
            draw.ellipse([30, 20, 70, 60], fill=(255, 200, 150))  # 头
            draw.rectangle([40, 60, 60, 120], fill=(100, 100, 200))  # 身体
            draw.rectangle([30, 120, 45, 180], fill=(50, 50, 150))  # 左腿
            draw.rectangle([55, 120, 70, 180], fill=(50, 50, 150))  # 右腿
            draw.rectangle([20, 70, 40, 100], fill=(100, 100, 200))  # 左臂
            draw.rectangle([60, 70, 80, 100], fill=(100, 100, 200))  # 右臂
            
            char.save(f"{self.assets_path}/characters/default.png")
        
        # 创建默认武器
        if not os.path.exists(f"{self.assets_path}/items/sword.png"):
            sword = Image.new('RGBA', (50, 100), (0, 0, 0, 0))
            draw = ImageDraw.Draw(sword)
            
            # 绘制简单的剑
            draw.rectangle([20, 10, 30, 70], fill=(200, 200, 200))  # 剑身
            draw.rectangle([10, 70, 40, 80], fill=(150, 100, 50))  # 剑柄
            
            sword.save(f"{self.assets_path}/items/sword.png")
    
    async def generate_battle_gif(self, battle_log, attacker, defender):
        """生成战斗GIF动画"""
        try:
            # 准备帧列表
            frames = []
            
            # 加载资源
            background = Image.open(f"{self.assets_path}/backgrounds/default.png")
            char_img = Image.open(f"{self.assets_path}/characters/default.png")
            
            # 尝试加载字体，如果失败则使用默认字体
            try:
                font = ImageFont.truetype("simhei.ttf", 20)
            except IOError:
                font = ImageFont.load_default()
            
            # 创建开场帧
            frame = background.copy()
            draw = ImageDraw.Draw(frame)
            
            # 绘制角色
            attacker_pos = (200, 300)
            defender_pos = (600, 300)
            
            frame.paste(char_img, attacker_pos, char_img)
            frame.paste(char_img, defender_pos, char_img)
            
            # 绘制名称和血量
            draw.text((attacker_pos[0], attacker_pos[1] - 30), f"{attacker['name']}", fill=(255, 255, 255), font=font)
            draw.text((attacker_pos[0], attacker_pos[1] - 60), f"HP: {attacker['hp']}", fill=(0, 255, 0), font=font)
            
            draw.text((defender_pos[0], defender_pos[1] - 30), f"{defender['name']}", fill=(255, 255, 255), font=font)
            draw.text((defender_pos[0], defender_pos[1] - 60), f"HP: {defender['hp']}", fill=(0, 255, 0), font=font)
            
            # 绘制VS文本
            draw.text((400, 250), "VS", fill=(255, 200, 0), font=ImageFont.truetype("simhei.ttf", 40) if 'simhei.ttf' in ImageFont.truetype.__code__.co_names else ImageFont.load_default())
            
            frames.append(frame)
            
            # 为每个战斗日志条目创建帧
            current_attacker_hp = attacker['hp']
            current_defender_hp = defender['hp']
            
            for i, log_entry in enumerate(battle_log):
                if "造成了" in log_entry and "点伤害" in log_entry:
                    # 解析伤害日志
                    parts = log_entry.split("对")
                    if len(parts) >= 2:
                        attacker_name = parts[0].strip()
                        rest = parts[1].split("造成了")
                        if len(rest) >= 2:
                            defender_name = rest[0].strip()
                            damage = int(rest[1].split("点伤害")[0].strip())
                            
                            # 更新血量
                            if attacker_name == attacker['name']:
                                current_defender_hp -= damage
                                current_defender_hp = max(0, current_defender_hp)
                                attacking_pos = attacker_pos
                                defending_pos = defender_pos
                            else:
                                current_attacker_hp -= damage
                                current_attacker_hp = max(0, current_attacker_hp)
                                attacking_pos = defender_pos
                                defending_pos = attacker_pos
                            
                            # 创建攻击帧
                            for j in range(5):  # 创建5帧的攻击动画
                                frame = background.copy()
                                draw = ImageDraw.Draw(frame)
                                
                                # 计算攻击者移动位置
                                if j < 3:
                                    # 向前移动
                                    attack_x = attacking_pos[0] + (defending_pos[0] - attacking_pos[0]) * j * 0.2
                                    attack_y = attacking_pos[1]
                                else:
                                    # 返回原位
                                    attack_x = attacking_pos[0] + (defending_pos[0] - attacking_pos[0]) * (5-j) * 0.2
                                    attack_y = attacking_pos[1]
                                
                                # 绘制攻击者
                                frame.paste(char_img, (int(attack_x), int(attack_y)), char_img)
                                
                                # 绘制防御者（可能会闪烁表示受伤）
                                if j == 2:  # 在攻击最前方时，防御者闪烁
                                    # 闪烁效果，跳过这一帧的防御者绘制
                                    pass
                                else:
                                    frame.paste(char_img, defending_pos, char_img)
                                
                                # 绘制名称和血量
                                draw.text((attacker_pos[0], attacker_pos[1] - 30), f"{attacker['name']}", fill=(255, 255, 255), font=font)
                                draw.text((attacker_pos[0], attacker_pos[1] - 60), f"HP: {current_attacker_hp}", fill=(0, 255, 0), font=font)
                                
                                draw.text((defender_pos[0], defender_pos[1] - 30), f"{defender['name']}", fill=(255, 255, 255), font=font)
                                draw.text((defender_pos[0], defender_pos[1] - 60), f"HP: {current_defender_hp}", fill=(0, 255, 0), font=font)
                                
                                # 绘制伤害文本
                                if j == 2:
                                    draw.text((defending_pos[0] + 20, defending_pos[1] - 100), f"-{damage}", fill=(255, 0, 0), font=font)
                                
                                # 绘制当前回合文本
                                draw.text((10, 10), f"回合: {i//2 + 1}", fill=(255, 255, 255), font=font)
                                
                                # 绘制战斗日志
                                draw.text((10, 550), log_entry, fill=(255, 255, 255), font=font)
                                
                                frames.append(frame)
                
                elif "被击败了" in log_entry:
                    # 创建结束帧
                    frame = background.copy()
                    draw = ImageDraw.Draw(frame)
                    
                    # 绘制角色（被击败的角色倒下）
                    if current_attacker_hp <= 0:
                        # 攻击者被击败，绘制倒下的攻击者
                        defeated_img = char_img.rotate(90)
                        frame.paste(defeated_img, (attacker_pos[0], attacker_pos[1] + 50), defeated_img)
                        frame.paste(char_img, defender_pos, char_img)
                    else:
                        # 防御者被击败，绘制倒下的防御者
                        frame.paste(char_img, attacker_pos, char_img)
                        defeated_img = char_img.rotate(90)
                        frame.paste(defeated_img, (defender_pos[0], defender_pos[1] + 50), defeated_img)
                    
                    # 绘制名称和血量
                    draw.text((attacker_pos[0], attacker_pos[1] - 30), f"{attacker['name']}", fill=(255, 255, 255), font=font)
                    draw.text((attacker_pos[0], attacker_pos[1] - 60), f"HP: {current_attacker_hp}", fill=(0, 255, 0), font=font)
                    
                    draw.text((defender_pos[0], defender_pos[1] - 30), f"{defender['name']}", fill=(255, 255, 255), font=font)
                    draw.text((defender_pos[0], defender_pos[1] - 60), f"HP: {current_defender_hp}", fill=(0, 255, 0), font=font)
                    
                    # 绘制战斗结果
                    winner_name = attacker['name'] if current_attacker_hp > 0 else defender['name']
                    draw.text((300, 100), f"{winner_name} 获胜!", fill=(255, 215, 0), font=ImageFont.truetype("simhei.ttf", 40) if 'simhei.ttf' in ImageFont.truetype.__code__.co_names else ImageFont.load_default())
                    
                    # 绘制战斗日志
                    draw.text((10, 550), log_entry, fill=(255, 255, 255), font=font)
                    
                    # 添加多个结束帧，让结束画面停留更长时间
                    for _ in range(10):
                        frames.append(frame)
            
            # 保存为GIF
            output_file = f"{self.output_path}/battle_{random.randint(1000, 9999)}.gif"
            
            # 使用imageio保存GIF
            imageio.mimsave(output_file, frames, duration=0.2)
            
            return output_file
        
        except Exception as e:
            print(f"生成战斗GIF时出错: {e}")