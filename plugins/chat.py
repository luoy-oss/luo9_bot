import sqlite3
import os, stat
import value


def Record(func):
    # __group_message_record__
    async def __group_message_record__(message_objects):
        if message_objects['message_type'] == 'group':
            message_time = message_objects['time']
            message = message_objects['message']
            group_id = message_objects['group_id']
            user_id =  message_objects['user_id']
            
            # 群文件夹
            data_path = value.data_path + '/{group_id}/'.format(group_id=group_id)
            group_path = data_path
            if not os.path.exists(data_path):
                os.makedirs(data_path)
                os.chmod(data_path, stat.S_IRWXO)
                
            data_path = group_path + '/chat_record.db'
            record_file = data_path
            if not os.path.isfile(data_path):
                conn = sqlite3.connect(data_path)
                cursor = conn.cursor()

                cursor.execute('''
                CREATE TABLE IF NOT EXISTS record (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    qq_id TEXT NOT NULL,
                    content TEXT NOT NULL,
                    time TEXT NOT NULL
                )
                ''')

                conn.commit()
                conn.close()
            else:
                conn = sqlite3.connect(record_file)
                cursor = conn.cursor()

                cursor.execute('''
                INSERT INTO record (qq_id, content, time)
                VALUES (?, ?, ?)
                ''', (user_id, message, message_time))

                conn.commit()
                conn.close()
        if message_objects['message_type'] == 'private':
            print("私聊消息")
        await func(message_objects)
    return __group_message_record__
