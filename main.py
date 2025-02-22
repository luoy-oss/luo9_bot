#main.py
import signal
import json

from flask import Flask, request
from luo9 import message_handle, notice_handle
from concurrent.futures import ThreadPoolExecutor
from plugins.schedule_task import schedule_run
from config import load_config, get_value

load_config('config.yaml')
value = get_value()


app = Flask(__name__)

@app.route('/', methods=['POST'])
async def receive_event():
    data = request.json
    if data['user_id'] == value.bot_id:
        print('机器人自身消息，进行阻断')
        return json.dumps({"OK": 200})
    # 消息事件
    if data['post_type'] == 'message':
        await message_handle(data)

    if data['post_type'] == 'notice':
        await notice_handle(data)

    return json.dumps({"OK": 200})

def run_flask():
    app.run(host=value.ncc_host, port=value.ncc_port)

def signal_handler(sig, frame):
    print('\r\n用户终止')
    exit(0)

if __name__ == '__main__':

    signal.signal(signal.SIGINT, signal_handler)
    with ThreadPoolExecutor() as executor:
        executor.submit(run_flask)
        schedule_run()

