from config import get_value
value = get_value()

from luo9_plugin_manager import PluginManager
plugin_manager = PluginManager(value.plugin_path)
