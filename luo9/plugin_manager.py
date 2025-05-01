from config import get_value
value = get_value()

import importlib
import os
import stat
import yaml
import platform
from luo9.message import Message, GroupMessage, PrivateMessage

from logger import Luo9Log
log = Luo9Log(__name__)

try:
    from luo9_plugin_manager import PluginManager
    log.info("成功导入Rust版本的插件管理器")
    USING_RUST_IMPLEMENTATION = True
except ImportError:
    def print_flip() -> None:
        print("---------------------------")

    class PluginManager:
        def __init__(self, plugin_dir):
            self.plugin_dir = plugin_dir
            self.plugins = []
            self.load_plugins()

        def load_plugins(self) -> None:
            config_path = os.path.join(self.plugin_dir, 'config.yaml')
            
            with open(config_path, 'r', encoding='utf-8') as f:
                config = yaml.safe_load(f)
            print("插件总数：", len(config['plugins']))
            print_flip()
            
            load_num = 0
            for plugin_config in config['plugins']:
                if plugin_config['enable']:
                    plugin_name = plugin_config['name']
                    plugin_path = os.path.join(self.plugin_dir, plugin_name)
                    if os.path.isdir(plugin_path):
                        plugin_module = importlib.import_module(f'plugins.{plugin_name}.main')
                        
                        plugin = {
                            'name': plugin_name,
                            'describe': plugin_module.config['describe'],
                            'author': plugin_module.config['author'],
                            'version': plugin_module.config['version'],
                            'module': plugin_module,
                            'priority': plugin_config['priority'],
                            'message_types': plugin_module.config['message_types']
                        }
                        
                        plugin_data_path = f"{value.data_path}/plugins/{plugin['name']}"
                        print(plugin_data_path)
                        if not os.path.exists(plugin_data_path):
                            os.makedirs(plugin_data_path)
                            if platform.system() != 'Windows':
                                os.chmod(plugin_data_path, stat.S_IRWXO)
                        log.info(f"加载插件：{plugin['name']}\n作者：{plugin['author']}\n插件描述：{plugin['describe']}\n插件版本：{plugin['version']}\n插件需求：{plugin_module.config['message_types']}")
                        print_flip()
                        load_num = load_num + 1
                        self.plugins.append(plugin)
            print(f"加载完成：{load_num}/{len(config['plugins'])}")
            print_flip()

        async def handle_group_message(self, message: GroupMessage) -> None:
            for plugin in sorted(self.plugins, key=lambda x: x['priority']):
                if 'group_message' in plugin['message_types']:
                    await plugin['module'].group_handle(message)

        async def handle_private_message(self, message: PrivateMessage) -> None:
            for plugin in sorted(self.plugins, key=lambda x: x['priority']):
                if 'private_message' in plugin['message_types']:
                    await plugin['module'].private_handle(message)

        async def handle_group_poke(self, target_id, user_id, group_id) -> None:
            for plugin in sorted(self.plugins, key=lambda x: x['priority']):
                if 'group_poke' in plugin['message_types']:
                    await plugin['module'].group_poke_handle(target_id, user_id, group_id)

    USING_RUST_IMPLEMENTATION = False
    log.warning("警告：Rust版本的插件管理器不可用，使用Python版本代替")

plugin_manager = PluginManager(value.plugin_path)
