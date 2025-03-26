from luo9.api_manager import luo9

class Ranking:
    """排行榜系统，显示玩家战力排名"""
    
    def __init__(self, db):
        """初始化排行榜系统"""
        self.db = db
    
    async def show_ranking(self, group_id, user_id, *args):
        """显示战力排行榜"""
        ranking_data = self.db.get_ranking(limit=10)
        
        if not ranking_data:
            await luo9.send_group_message(group_id, "暂无排行榜数据")
            return
        
        # 构建排行榜消息
        ranking_message = "【战力排行榜】\n"
        
        for i, user in enumerate(ranking_data):
            # 计算战力
            power = self.db.calculate_user_power(user['user_id'])
            
            # 添加排名标记
            rank_mark = "🥇" if i == 0 else "🥈" if i == 1 else "🥉" if i == 2 else f"{i+1}."
            
            # 如果是当前用户，添加标记
            user_mark = "👉 " if user['user_id'] == user_id else ""
            
            ranking_message += f"{user_mark}{rank_mark} {user['name']} - 战力:{power} (Lv.{user['level']})\n"
        
        # 如果当前用户不在前10名，添加用户自己的排名
        if not any(user['user_id'] == user_id for user in ranking_data):
            # 获取用户排名
            conn = self.db.conn
            cursor = conn.cursor()
            
            cursor.execute('''
            SELECT COUNT(*) + 1 as rank
            FROM users
            WHERE (attack + defense + hp/10 + speed + (wins - losses) * 5) > 
                  (SELECT (attack + defense + hp/10 + speed + (wins - losses) * 5) FROM users WHERE user_id = ?)
            ''', (user_id,))
            
            user_rank = cursor.fetchone()[0]
            user_stats = self.db.get_user_battle_stats(user_id)
            
            if user_stats:
                ranking_message += f"\n你的排名: 第{user_rank}名 - 战力:{user_stats['power']} (Lv.{user_stats['level']})"
        
        await luo9.send_group_message(group_id, ranking_message)