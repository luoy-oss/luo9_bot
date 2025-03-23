import asyncio
import time
import os
from typing import Dict, Any, List, Optional, Callable, Coroutine
from luo9.message import GroupMessage, PrivateMessage
from luo9.api_manager import luo9
from config import get_value

value = get_value()

class MockMessage:
    """模拟消息对象"""
    def __init__(self, content: str, user_id: int = 10000, group_id: Optional[int] = None, 
                 user_name: str = "测试用户"):
        self.content = content
        self.user_id = user_id
        self.group_id = group_id
        self.user_name = user_name
        self.time = int(time.time())
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        result = {
            "message": self.content,
            "user_id": self.user_id,
            "sender": {"nickname": self.user_name},
            "time": self.time
        }
        if self.group_id:
            result["group_id"] = self.group_id
        return result

class MockAPI:
    """模拟API调用"""
    def __init__(self):
        self.sent_messages: List[Dict[str, Any]] = []
    
    async def send_group_message(self, group_id: int, message: str):
        """模拟发送群消息"""
        print(f"\n[发送到群 {group_id}] {message}")
        self.sent_messages.append({
            "type": "group",
            "target_id": group_id,
            "content": message,
            "time": time.strftime("%Y-%m-%d %H:%M:%S")
        })
        return {"status": "ok"}
    
    async def send_private_msg(self, user_id: int, message: str):
        """模拟发送私聊消息"""
        print(f"\n[发送到用户 {user_id}] {message}")
        self.sent_messages.append({
            "type": "private",
            "target_id": user_id,
            "content": message,
            "time": time.strftime("%Y-%m-%d %H:%M:%S")
        })
        return {"status": "ok"}
    
    def get_sent_messages(self) -> List[Dict[str, Any]]:
        """获取已发送的消息"""
        return self.sent_messages
    
    def clear_sent_messages(self):
        """清空已发送的消息"""
        self.sent_messages = []

class PluginDebugger:
    """插件调试器"""
    def __init__(self, plugin_name: str):
        self.plugin_name = plugin_name
        self.mock_api = MockAPI()
        self.original_api = None
        self.plugin_module = None
        
    async def load_plugin(self):
        """加载插件"""
        try:
            # 确保数据目录存在
            os.makedirs(f"{value.data_path}/plugins/{self.plugin_name}", exist_ok=True)
            
            # 动态导入插件
            plugin_path = f"plugins.{self.plugin_name}.main"
            import importlib
            self.plugin_module = importlib.import_module(plugin_path)
            print(f"插件 {self.plugin_name} 加载成功")
            return True
        except Exception as e:
            print(f"加载插件失败: {e}")
            return False
    
    def patch_api(self):
        """替换API为模拟API"""
        self.original_api = luo9.__dict__.copy()
        for method_name in dir(self.mock_api):
            if not method_name.startswith('_') and callable(getattr(self.mock_api, method_name)):
                setattr(luo9, method_name, getattr(self.mock_api, method_name))
    
    def restore_api(self):
        """恢复原始API"""
        if self.original_api:
            for key, value in self.original_api.items():
                setattr(luo9, key, value)
    
    async def simulate_group_message(self, content: str, group_id: int = 12345, 
                                    user_id: int = 10000, user_name: str = "测试用户"):
        """模拟群消息"""
        mock_msg = MockMessage(content, user_id, group_id, user_name)
        msg_dict = mock_msg.to_dict()
        
        # 创建GroupMessage对象
        group_msg = GroupMessage()
        group_msg.handle(msg_dict)
        
        # 调用插件的群消息处理函数
        try:
            await self.plugin_module.group_handle(group_msg)
        except Exception as e:
            import traceback
            print(f"处理群消息时出错: {e}")
            print(traceback.format_exc())
        else:
            print("插件没有实现群消息处理函数")
    
    async def simulate_private_message(self, content: str, user_id: int = 10000, 
                                      user_name: str = "测试用户"):
        """模拟私聊消息"""
        mock_msg = MockMessage(content, user_id, None, user_name)
        msg_dict = mock_msg.to_dict()
        
        # 创建PrivateMessage对象
        private_msg = PrivateMessage()
        private_msg.handle(msg_dict)
        
        # 调用插件的私聊消息处理函数
        if hasattr(self.plugin_module, 'private_handle'):
            print(f"\n[接收私聊消息] {content}")
            await self.plugin_module.private_handle(private_msg)
        else:
            print("插件没有实现私聊消息处理函数")
    
    async def run_startup(self):
        """运行插件的启动函数"""
        if hasattr(self.plugin_module, 'startup'):
            print("运行插件启动函数...")
            await self.plugin_module.startup()
        else:
            print("插件没有实现启动函数")
    
    async def run_shutdown(self):
        """运行插件的关闭函数"""
        if hasattr(self.plugin_module, 'shutdown'):
            print("运行插件关闭函数...")
            await self.plugin_module.shutdown()
        else:
            print("插件没有实现关闭函数")
    
    def get_sent_messages(self):
        """获取已发送的消息"""
        return self.mock_api.get_sent_messages()
    
    def clear_sent_messages(self):
        """清空已发送的消息"""
        self.mock_api.clear_sent_messages()

async def run_task_once(task_func: Callable[[], Coroutine]):
    """运行一次定时任务"""
    try:
        await task_func()
        return True
    except Exception as e:
        print(f"运行任务失败: {e}")
        return False

async def debug_plugin(plugin_name: str):
    """调试插件的主函数"""
    debugger = PluginDebugger(plugin_name)
    
    if not await debugger.load_plugin():
        return
    
    debugger.patch_api()
    await debugger.run_startup()
    
    print(f"\n===== {plugin_name} 调试模式 =====")
    print("输入 'exit' 退出调试")
    print("输入 'group <消息>' 发送群消息")
    print("输入 'private <消息>' 发送私聊消息")
    print("输入 'task <任务名>' 运行一次定时任务")
    print("输入 'clear' 清空已发送的消息")
    print("输入 'help' 显示帮助")
    
    try:
        while True:
            cmd = input("\n> ")
            if cmd.lower() == 'exit':
                break
            elif cmd.lower() == 'help':
                print("命令列表:")
                print("  exit - 退出调试")
                print("  group <消息> - 发送群消息")
                print("  private <消息> - 发送私聊消息")
                print("  task <任务名> - 运行一次定时任务")
                print("  clear - 清空已发送的消息")
                print("  help - 显示帮助")
            elif cmd.lower() == 'clear':
                debugger.clear_sent_messages()
                print("已清空消息记录")
            elif cmd.lower().startswith('group '):
                msg = cmd[6:]
                await debugger.simulate_group_message(msg)
            elif cmd.lower().startswith('private '):
                msg = cmd[8:]
                await debugger.simulate_private_message(msg)
            elif cmd.lower().startswith('task '):
                task_name = cmd[5:]
                if hasattr(debugger.plugin_module, task_name):
                    task_func = getattr(debugger.plugin_module, task_name)
                    if callable(task_func):
                        print(f"运行任务 {task_name}...")
                        success = await run_task_once(task_func)
                        if success:
                            print(f"任务 {task_name} 运行完成")
                    else:
                        print(f"{task_name} 不是一个可调用的函数")
                else:
                    print(f"找不到任务 {task_name}")
            else:
                print("未知命令，输入 'help' 查看帮助")
    except KeyboardInterrupt:
        print("\n用户中断")
    finally:
        await debugger.run_shutdown()
        debugger.restore_api()
        print(f"\n===== {plugin_name} 调试结束 =====")