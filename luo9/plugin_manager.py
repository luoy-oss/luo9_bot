import importlib
import os
import yaml
from pathlib import Path

def print_flip() -> None:
    print("---------------------------")


class PluginManager:
    def __init__(self, plugin_dir):
        self.plugin_dir = plugin_dir
        self.plugins = []
        self.load_plugins()

    def load_plugins(self):
        config_path = os.path.join(self.plugin_dir, 'config.yaml')
        
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
        print("插件总数：", len(config['plugins']))
        print_flip()
        
        load_num = 0
        for plugin_config in config['plugins']:
            if plugin_config['enabled']:
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
                    print(f"加载插件：{plugin['name']}\n作者：{plugin['author']}\n插件描述：{plugin['describe']}\n插件版本：{plugin['version']}\n插件需求：{plugin_module.config['message_types']}")
                    print_flip()
                    load_num = load_num + 1
                    self.plugins.append(plugin)
        print(f"加载完成：{load_num}/{len(config['plugins'])}")
        print_flip()

    async def handle_group_message(self, message, group_id, user_id):
        for plugin in sorted(self.plugins, key=lambda x: x['priority']):
            if 'group' in plugin['message_types']:
                # pass
                await plugin['module'].group_handle(message, group_id, user_id)

    async def handle_private_message(self, message, user_id):
        for plugin in sorted(self.plugins, key=lambda x: x['priority']):
            if 'private' in plugin['message_types']:
                await plugin['module'].private_handle(message, user_id)

from config import get_value
value = get_value()

plugin_manager = PluginManager(value.plugin_path)
