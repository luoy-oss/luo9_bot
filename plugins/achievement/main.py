import utils
import luo9
import value
import os, stat
import sqlite3

async def get_achievement(group_id, qq, path):
    USER_DATA_PATH = path['USER_DATA_PATH']

    if await utils.interactiveState_check():
        msg = '[CQ:at,qq={qq}]\n'.format(qq=qq)
        if await utils.register_check(group_id, qq, USER_DATA_PATH):
            msg += '———成就列表———\n'

            data_path = value.data_path + '/Achievement/'
            achieve_path = data_path
            if not os.path.exists(data_path):
                os.makedirs(data_path)
                os.chmod(data_path, stat.S_IRWXO)
                
            data_path = achieve_path + '/achievements.db'
            record_file = data_path

            conn = sqlite3.connect(record_file)
            cursor = conn.cursor()

            cursor.execute('''
            CREATE TABLE IF NOT EXISTS '{user_id}' (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                achieve TEXT NOT NULL,
                datetime TEXT NOT NULL,
                remark TEXT NOT NULL
            )
            '''.format(user_id=qq))

            data = cursor.execute('''
                SELECT achieve, datetime, remark FROM '{user_id}'
                '''.format(user_id=qq)).fetchall()
    
            
            if len(data) > 0:
                for achieve, datetime, remark in data:
                    msg += achieve
                    if datetime != '':
                       msg += '-' + datetime
                    msg += '\n'
            else:
                pass

            conn.commit()
            conn.close()

        else:
            msg += '你还没有注册哦，请先注册'

        msg += '______________________'
        await luo9.send_group_message(group_id, msg)
    else:
        pass
