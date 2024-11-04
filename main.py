#main.py
import value
import signal

from flask import Flask, request
from luo9 import message_handle, notice_handle
from concurrent.futures import ThreadPoolExecutor
from plugins.schedule_task import schedule_run

app = Flask(__name__)

@app.route('/', methods=['POST'])
async def receive_event():
    data = request.json
    if data['user_id'] == value.bot_id:
        print('机器人自身消息，进行阻断')
        return "OK", 200
    # 消息事件
    if data['post_type'] == 'message':
        await message_handle(data)

    if data['post_type'] == 'notice':
        await notice_handle(data)

    return "OK", 200

def run_flask():
    app.run(host='0.0.0.0', port=7777)

def signal_handler(sig, frame):
    print('\r\n用户终止')
    exit(0)

if __name__ == '__main__':
    signal.signal(signal.SIGINT, signal_handler)
    with ThreadPoolExecutor() as executor:
        executor.submit(run_flask)
        schedule_run()

