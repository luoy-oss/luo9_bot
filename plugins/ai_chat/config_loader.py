import os
import warnings
import yaml
from logger import Luo9Log
from openai import OpenAI
from config import get_value

value = get_value()

def load_config(path, file_name):
    config_path = os.path.join(path, file_name)
    try:
        with open(config_path, "r", encoding="utf-8") as f:
            config = yaml.safe_load(f)
    except FileNotFoundError:
        warnings.warn(
            f"在{path}目录中未找到文件{file_name} 使用默认样例config.(example).yaml配置\n请参考默认样例config.(example).yaml，在本插件目录中创建config.yaml进行配置"
        )
        config_path = os.path.join(path, "config.(example).yaml")
        with open(config_path, "r", encoding="utf-8") as f:
            config = yaml.safe_load(f)
    return config

ai_config = load_config(value.plugin_path + "/ai_chat", "config.yaml")

client = OpenAI(
    api_key=ai_config["DEEPSEEK_API_KEY"],
    base_url=ai_config["DEEPSEEK_BASE_URL"],
    default_headers={"Content-Type": "application/json"},
)

root_dir = os.path.dirname(os.path.abspath(__file__))
file_path = os.path.join(root_dir, "prompts", ai_config["prompts"])

try:
    with open(file_path, "r", encoding="utf-8") as file:
        prompt_content = file.read()
except FileNotFoundError:
    warnings.warn(
        f"在{file_path}目录中未找到prompt文件：{ai_config['prompts']} 使用默认提示词文件default_prompts.txt配置\n请参考default_prompts.txt，在同级目录中创建你的提示词文件，并在插件config中设置prompts参数"
    )
    file_path = os.path.join(root_dir, "prompts", "default_prompts.txt")
    with open(file_path, "r", encoding="utf-8") as file:
        prompt_content = file.read()

logger = Luo9Log(__name__)
