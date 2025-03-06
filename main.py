from config import load_config, get_value
load_config('config.yaml')
value = get_value()

#main.py
import signal
import json
import asyncio

from luo9 import get_driver
from flask import Flask, request
from luo9.message import message_handle
from luo9.notice import notice_handle
from concurrent.futures import ThreadPoolExecutor

driver = get_driver()
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
    from waitress import serve
    serve(app, host=value.ncc_host, port=value.ncc_port)

async def startup():
    await driver.run_startup()

async def shutdown():
    await driver.run_shutdown()

def signal_handler(sig, frame):
    asyncio.run(shutdown())
    print('\r\n用户终止')
    exit(0)

from luo9 import get_task
task = get_task()

async def run_task():
    await task.start()

if __name__ == '__main__':
    asyncio.run(startup())
    # asyncio.run(run_task())
    signal.signal(signal.SIGINT, signal_handler)
    with ThreadPoolExecutor() as executor:
        executor.submit(run_flask)
        asyncio.run(run_task())
