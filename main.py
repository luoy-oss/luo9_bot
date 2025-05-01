from config import load_config, get_value
load_config('config.yaml')
value = get_value()

#main.py
import os
import signal
import json
import asyncio
from utils import data_encode

from luo9 import get_driver
from flask import Flask, request, current_app
from luo9.handle import message_handle, notice_handle
from concurrent.futures import ThreadPoolExecutor
from logger import Luo9Log
log = Luo9Log(__name__)

driver = get_driver()
app = Flask(__name__)


@app.route('/', methods=['POST'])
async def receive_event():
    data = data_encode(request.json)
    
    if data['user_id'] == value.bot_id:
        log.info('机器人自身消息，进行阻断')
        return json.dumps({"OK": 200})
    # 消息事件
    if data['post_type'] == 'message':
        await message_handle(data)

    if data['post_type'] == 'notice':
        await notice_handle(data)

    return json.dumps({"OK": 200})

def run_flask():
    from waitress import serve
    log.info(f'>>>>>>> 启动Flask服务，监听 {value.ncc_host}:{value.ncc_port} <<<<<<<')
    serve(app, host=value.ncc_host, port=value.ncc_port)

def signal_handler(sig, frame):
    log.info('>>>>>>> 收到终止信号，关闭程序 <<<<<<<')
    driver.run_shutdown()
    executor.shutdown(wait=False)
    app.do_teardown_appcontext()
    import time
    time.sleep(1)
    os.kill(os.getpid(), signal.SIGTERM)
    
from luo9 import get_task
task = get_task()

async def run_task():
    await task.start()

if __name__ == '__main__':
    driver.run_startup()
    
    signal.signal(signal.SIGINT, signal_handler)
    with ThreadPoolExecutor() as executor:
        executor.submit(run_flask)
        asyncio.run(run_task())
