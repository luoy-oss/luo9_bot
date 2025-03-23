from config import load_config, get_value
load_config('config.yaml')

import asyncio
import sys
import os
from utils.debug import debug_plugin


async def main():
    if len(sys.argv) < 2:
        print("用法: python debug.py <插件名称>")
        print("\n可用插件:")
        plugins_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "plugins")
        for item in os.listdir(plugins_dir):
            if os.path.isdir(os.path.join(plugins_dir, item)) and not item.startswith('__'):
                print(f"  - {item}")
        return
    
    plugin_name = sys.argv[1]
    await debug_plugin(plugin_name)

if __name__ == "__main__":
    asyncio.run(main())