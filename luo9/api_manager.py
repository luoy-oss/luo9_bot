import importlib
import os
import warnings
from config import get_value
value = get_value()

class APIManager:
    def __init__(self, api_name):
        self.api_name = api_name
        self.group = None
        self.private = None
        self.__load_api()

    def __load_api(self):
        api_path = os.path.join('./luo9', self.api_name)
        
        if os.path.isdir(api_path):
            # 动态导入 group.py 模块
            try:
                self.group = importlib.import_module(f'luo9.{self.api_name}.group')
            except ModuleNotFoundError:
                warnings.warn(f"API '{self.api_name}.group' not found in luo9.{self.api_name}")

            try:
                self.private = importlib.import_module(f'luo9.{self.api_name}.user')
            except ModuleNotFoundError:
                warnings.warn(f"API '{self.api_name}.user' not found in luo9.{self.api_name}")

        else:
            raise ImportError(f"API 'luo9.{self.api_name}' not found in luo9")

    async def _call_private_method(self, method_name, *args, **kwargs):
        if self.private:
            method = getattr(self.private, method_name, None)
            if method:
                return await method(*args, **kwargs)
            else:
                warnings.warn(f"API 'luo9.{self.api_name}.user' has no attribute '{method_name}'")
        else:
            raise RuntimeError(f"{self.api_name} API not loaded user package")
    
    async def _call_group_method(self, method_name, *args, **kwargs):
        if self.group:
            method = getattr(self.group, method_name, None)
            if method:
                return await method(*args, **kwargs)
            else:
                warnings.warn(f"API 'luo9.{self.api_name}.group' has no attribute '{method_name}'")
        else:
            raise RuntimeError(f"{self.api_name} API not loaded group package")
    
    # 群聊消息
    async def send_group_message(self, group_id : str, message : str, ignore=True):
        await self._call_group_method('send_group_message', group_id, message)

    # 群聊消息
    async def send_group_ai_record(self, group_id : str, voice : str, message : str):
        await self._call_group_method('send_group_ai_record', group_id, voice, message)

    # 群聊AT
    async def send_group_at(self, group_id : str, qq : str):
        await self._call_group_method('send_group_at', group_id, qq)

    # 群聊图片
    async def send_group_image(self, group_id : str, file : str):
        await self._call_group_method('send_group_image', group_id, file)
    
    # 群聊文件
    async def send_group_file(self, group_id : str, file : str, name : str, folder_id : str):
        await self._call_group_method('upload_group_file', group_id, file, name, folder_id)

    # 群聊戳一戳
    async def send_group_poke(self, group_id : str, user_id : str):
        await self._call_group_method('send_group_poke', group_id, user_id)

    # 私聊消息
    async def send_private_msg(self, user_id : str, message : str):
        await self._call_private_method('send_private_msg', user_id, message)

    async def get_group_files_by_folder(self, group_id : str, folder_id : str, file_count : int):
        ret = await self._call_group_method('get_group_files_by_folder', group_id, folder_id, file_count)
        return ret

    async def get_group_root_files(self, group_id : str):
        ret = await self._call_group_method('get_group_root_files', group_id)
        return ret

# luo9 =  APIManager(api_name="NapCat")
if value.napcat:
    luo9 = APIManager(api_name="NapCat")

# if value.qqbot:
#     luo9 = APIManager(api_name="QQBot")