# 洛玖机器人

> 推荐使用py3.10.12

## 运行

> 克隆本仓库
```
git clone https://github.com/luoy-oss/luo9_bot.git
```

> 进入bot主目录
```
cd luo9_bot
```

> 运行
>
> **运行前请进行基础config配置**
```
sudo python3 main.py
```

## 基础配置

> 参考config.(example).yaml
> 
> 在同级目录下添加文件：config.yaml进行配置

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

> main.py 必须含有：
```python
# 插件信息
config = {
    'name': 'core_plugin',
    'describe': '签到注册模块',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group']
}

# message_types 支持 group, private
# 当前未编写private消息处理部分，请留意

# 对于message_types 含有 group，你必须同步编写以下函数用于接受传递给插件的群消息：
async def group_handle(message, group_id, user_id):
    pass
# 其中message为群消息内容，group_id为群号，user_id为用户qq号

# 对于message_types 含有 private，你必须同步编写：
async def private_handle(message, group_id, user_id):
    pass

# 你可以在插件代码中通过以下形式，获取全局config.yaml中配置的参数
# 利用value.xxxx进行调用
from config import get_value
value = get_value()

```


# 免责声明

代码仅用于对Python技术的交流学习使用，禁止用于实际生产项目，请勿用于非法用途和商业用途！如因此产生任何法律纠纷，均与作者无关！