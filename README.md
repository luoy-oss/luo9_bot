<div align="center">

# 洛玖机器人

_✨ 推荐使用python 3.10.12 ✨_

<a href="https://raw.githubusercontent.com/luoy-oss/luo9_bot/main/LICENSE">
    <img src="https://img.shields.io/github/license/luoy-oss/luo9_bot" alt="license">
</a>
<!-- <a href="https://pypi.python.org/pypi/packageName">
    <img src="https://img.shields.io/pypi/v/packageName?logo=python&logoColor=edb641" alt="pypi">
</a> -->
<img src="https://img.shields.io/badge/python-3.10+-blue?logo=python&logoColor=edb641" alt="python">
<a href="https://github.com/astral-sh/ruff">
    <img src="https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/charliermarsh/ruff/main/assets/badge/v2.json" alt="ruff">
</a>
<br />

<!-- <a href="https://github.com/luoy-oss/luo9_bot/actions/workflows/ruff.yml">
    <img src="https://github.com/luoy-oss/luo9_bot/actions/workflows/ruff.yml/badge.svg?branch=main&event=push" alt="ruff">
</a> -->

<br />

</div>

## 运行

> 克隆本仓库
```
git clone https://github.com/luoy-oss/luo9_bot.git
```

<br>

> 进入bot主目录
```
cd luo9_bot
```

<br>

> 安装依赖
```
pip install -r requiremes.txt
```

<br>

> 运行
>
> **运行前请进行基础config配置**
```
python3 main.py
```

<br>

## 基础配置

> 参考config.(example).yaml
> 
> 在同级目录下添加文件：config.yaml进行配置

<br>

## 插件编写

> 在plugins目录下新建你的插件
- plugins
    - your_plugin_folder
        - main.py(必需)
        - xxx.py(其余辅助文件)
> 向plugins/config.yaml添加插件信息
>
> name为插件文件夹名称(插件名称)
> 
> priority为插件优先级，建议值：1-65535，值越小，插件优先级越高
> 
> enable为插件是否启用，true为启用
```yaml
  - name: your_plugin_folder
    priority: 1
    enable: false
```

<br>

> main.py 必须含有：
```python
# 插件信息
config = {
    'name': 'plugin_name',
    'describe': '插件描述',
    'author': '插件作者',
    'version': '插件版本',
    'message_types': ['group_message', 'private_message', 'group_poke']
}

# message_types 支持 group, private
# 当前未编写private消息处理部分，请留意

# 对于message_types 含有 group_message，你必须同步编写以下函数用于接受传递给插件的群消息：
async def group_handle(message, group_id, user_id):
    pass
# 其中message为群消息内容，group_id为群号，user_id为用户qq号

# 对于message_types 含有 group_poke，你必须同步编写以下函数用于接受传递给插件的群消息：
async def group_poke_handle(target_id, user_id, group_id):
    pass
# 对于A用户戳了一下B用户
# 其中target_id为B用户，user_id为A用户，group_id为群号

# 对于message_types 含有 private_message，你必须同步编写：
async def private_handle(message, user_id):
    pass

# 你可以在插件代码中通过以下形式，获取全局config.yaml中配置的参数
# 利用value.xxxx进行调用
from config import get_value
value = get_value()

```

<br>

## 启动前后处理

你可以通过

driver中的on_startup与on_shutdown装饰器

进行bot启动前/停止前插件的信息处理

```python
from luo9 import get_driver

driver = get_driver()

@driver.on_startup
async def _():
    print("插件启动前处理")


@driver.on_shutdown
async def _():
    print("插件停止前处理")

async def group_handle(message, group_id, user_id):
    pass
# 其余插件代码

```

## 计划任务

你可以通过

task中的on_schedule_task装饰器，参考AsyncIOScheduler计划任务规则

你可以使用task中的adjust_interval装饰器，调整计划任务的定时时间，参数参考AsyncIOScheduler计划任务规则

进行bot计划任务

```python
from luo9 import get_task

task = get_task()

config = {
    'name': 'schedule_task',
    'describe': '定时任务',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['']
}

@task.on_schedule_task(trigger ='interval', minutes=1)
async def task1():
    print("每分钟调用一次")

@task.on_schedule_task(trigger ='cron', second=0, minute=0, hour=6)
async def task2():
    print("每天06:00:00（6时0分0秒）调用一次")

task3_status = 0
@task.on_schedule_task(trigger ='interval', minutes=1)
async def task3():
    # 格外注意,在函数内对变量进行全局声明
    global task3_status
    if task3_status == 0:
        task3_status = 1
        # 修改任务状态为2分钟调用一次
        task.adjust_interval(task3, 'interval', minutes=2)

    if task3_status == 1:
        task3_status = 0
        # 修改任务状态为1分钟调用一次
        task.adjust_interval(task3, 'interval', minutes=1)



# 其余插件代码

```


# 免责声明

代码仅用于对Python技术的交流学习使用，禁止用于实际生产项目，请勿用于非法用途和商业用途！如因此产生任何法律纠纷，均与作者无关！