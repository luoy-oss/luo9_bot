#main.py
from flask import Flask, request
import plugins
import utils
import value
from luo9 import message_handle, notice_handle

app = Flask(__name__)

@app.route('/', methods=['POST'])
async def receive_event():
    data = request.json
    if str(data['user_id']) == str(value.bot_id):
        print('机器人自身消息，进行阻断')
        return "OK", 200
    # 消息事件
    if data['post_type'] == 'message':
        await message_handle(data)

    if data['post_type'] == 'notice':
        await notice_handle(data)

    return "OK", 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=7777)