from config import load_config, get_value
load_config('config.yaml')

import os
import sys
import json
import time
import asyncio
import argparse
import threading
import webbrowser
from datetime import datetime
from flask import Flask, request, jsonify, render_template, send_from_directory
from flask_socketio import SocketIO, emit
from flask_cors import CORS

from luo9 import get_driver
from luo9.handle import message_handle, notice_handle
value = get_value()

# 初始化Flask应用
app = Flask(__name__, 
            static_folder='debug_ui/static',
            template_folder='debug_ui/templates')
CORS(app)
socketio = SocketIO(app, cors_allowed_origins="*")

# 初始化驱动
driver = get_driver()

# 调试状态
debug_state = {
    "plugin_name": "",
    "active": False,
    "history": {},
    "current_session": []
}

# 历史记录存储路径
HISTORY_DIR = os.path.join(value.data_path, "debug_history")
os.makedirs(HISTORY_DIR, exist_ok=True)

# 模拟用户ID和群ID
DEFAULT_USER_ID = 10000
DEFAULT_GROUP_ID = 10000

def save_history():
    """保存当前会话历史到文件"""
    if not debug_state["plugin_name"] or not debug_state["current_session"]:
        return
    
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"{debug_state['plugin_name']}_{timestamp}.json"
    filepath = os.path.join(HISTORY_DIR, filename)
    
    with open(filepath, 'w', encoding='utf-8') as f:
        json.dump(debug_state["current_session"], f, ensure_ascii=False, indent=2)
    
    # 更新历史记录索引
    if debug_state["plugin_name"] not in debug_state["history"]:
        debug_state["history"][debug_state["plugin_name"]] = []
    
    debug_state["history"][debug_state["plugin_name"]].append({
        "timestamp": timestamp,
        "filename": filename,
        "message_count": len(debug_state["current_session"])
    })
    
    # 保存历史记录索引
    history_index_path = os.path.join(HISTORY_DIR, "history_index.json")
    with open(history_index_path, 'w', encoding='utf-8') as f:
        json.dump(debug_state["history"], f, ensure_ascii=False, indent=2)

def load_history():
    """加载历史记录索引"""
    history_index_path = os.path.join(HISTORY_DIR, "history_index.json")
    if os.path.exists(history_index_path):
        with open(history_index_path, 'r', encoding='utf-8') as f:
            debug_state["history"] = json.load(f)

def load_session_history(filename):
    """加载特定会话的历史记录"""
    filepath = os.path.join(HISTORY_DIR, filename)
    if os.path.exists(filepath):
        with open(filepath, 'r', encoding='utf-8') as f:
            return json.load(f)
    return []

# 模拟消息处理
async def handle_simulated_message(message_data):
    """处理模拟的消息"""
    # 记录用户发送的消息
    timestamp = time.time()
    formatted_time = datetime.fromtimestamp(timestamp).strftime("%Y-%m-%d %H:%M:%S")
    
    message_record = {
        "id": len(debug_state["current_session"]) + 1,
        "timestamp": timestamp,
        "formatted_time": formatted_time,
        "direction": "outgoing",  # 用户发出的消息
        "content": message_data["content"],
        "type": message_data.get("type", "text"),
        "user_id": message_data.get("user_id", DEFAULT_USER_ID),
        "group_id": message_data.get("group_id", DEFAULT_GROUP_ID)
    }
    
    debug_state["current_session"].append(message_record)
    socketio.emit('message_update', message_record)
    
    # 构造消息对象
    if message_data.get("message_type") == "private":
        simulated_data = {
            "post_type": "message",
            "message_type": "private",
            "user_id": message_data.get("user_id", DEFAULT_USER_ID),
            "message": message_data["content"],
            "raw_message": message_data["content"],
            "time": int(timestamp),
            "self_id": value.bot_id,
            # 添加sender信息
            "sender": {
                "user_id": message_data.get("user_id", DEFAULT_USER_ID),
                "nickname": f"测试用户_{message_data.get('user_id', DEFAULT_USER_ID)}",
                "card": "",
                "role": "member"
            }
        }
    else:  # 默认为群消息
        simulated_data = {
            "post_type": "message",
            "message_type": "group",
            "group_id": message_data.get("group_id", DEFAULT_GROUP_ID),
            "user_id": message_data.get("user_id", DEFAULT_USER_ID),
            "message": message_data["content"],
            "raw_message": message_data["content"],
            "time": int(timestamp),
            "self_id": value.bot_id,
            # 添加sender信息
            "sender": {
                "user_id": message_data.get("user_id", DEFAULT_USER_ID),
                "nickname": f"测试用户_{message_data.get('user_id', DEFAULT_USER_ID)}",
                "card": "",
                "role": "member"
            }
        }
    
    # 拦截发送函数的响应
    def message_interceptor(response_data):
        """拦截并记录机器人的响应"""
        timestamp = time.time()
        formatted_time = datetime.fromtimestamp(timestamp).strftime("%Y-%m-%d %H:%M:%S")
        
        response_record = {
            "id": len(debug_state["current_session"]) + 1,
            "timestamp": timestamp,
            "formatted_time": formatted_time,
            "direction": "incoming",  # 机器人发来的消息
            "content": response_data.get("content", ""),
            "type": response_data.get("type", "text"),
            "user_id": value.bot_id,
            "group_id": message_data.get("group_id", DEFAULT_GROUP_ID)
        }
        
        debug_state["current_session"].append(response_record)
        socketio.emit('message_update', response_record)
    
    # 设置消息拦截器
    from luo9.api_manager import luo9
    original_send_group_message = luo9.send_group_message
    original_send_group_image = luo9.send_group_image
    original_send_private_msg = luo9.send_private_msg
    
    async def intercepted_send_group_message(group_id, message, ignore=True):
        message_interceptor({"content": message, "type": "text"})
        # 不实际发送消息，仅模拟
    
    async def intercepted_send_group_image(group_id, file):
        # 检查文件是否存在
        if os.path.exists(file):
            # 如果是绝对路径，复制到uploads目录
            filename = os.path.basename(file)
            uploads_dir = os.path.join(value.data_path, "uploads")
            os.makedirs(uploads_dir, exist_ok=True)
            target_path = os.path.join(uploads_dir, filename)
            
            # 复制文件
            import shutil
            shutil.copy2(file, target_path)
            
            # 只使用文件名进行消息拦截
            message_interceptor({"content": filename, "type": "image"})
        else:
            # 如果只是文件名，假设它已经在uploads目录中
            message_interceptor({"content": file, "type": "image"})
        
        # 不实际发送图片，仅模拟
    
    async def intercepted_send_private_msg(user_id, message):
        message_interceptor({"content": message, "type": "text"})
        # 不实际发送私聊消息，仅模拟
    
    # 替换发送函数
    luo9.send_group_message = intercepted_send_group_message
    luo9.send_group_image = intercepted_send_group_image
    luo9.send_private_msg = intercepted_send_private_msg
    
    try:
        # 处理消息
        await message_handle(simulated_data)
    except Exception as e:
        error_message = f"处理消息时出错: {str(e)}"
        socketio.emit('error', {"message": error_message})
        print(error_message)
    finally:
        # 恢复原始发送函数
        luo9.send_group_message = original_send_group_message
        luo9.send_group_image = original_send_group_image
        luo9.send_private_msg = original_send_private_msg

# Flask路由
@app.route('/')
def index():
    """渲染调试界面"""
    return render_template('index.html')

# 全局变量，用于存储预加载的插件信息
preloaded_plugins = []
plugins_loaded = False  # 新增标志，用于跟踪插件是否已加载

def preload_plugins():
    """预加载所有插件信息"""
    global preloaded_plugins, plugins_loaded
    
    # 如果插件已加载且不是强制刷新，则直接返回
    if plugins_loaded:
        return
    
    try:
        plugin_dir = value.plugin_path
        preloaded_plugins = []  # 清空现有插件列表
        
        print(f"正在预加载插件信息，从 {plugin_dir} ...")
        
        # 遍历插件目录
        for plugin_name in os.listdir(plugin_dir):
            plugin_path = os.path.join(plugin_dir, plugin_name)
            
            # 检查是否是目录且包含 main.py
            if os.path.isdir(plugin_path) and os.path.exists(os.path.join(plugin_path, 'main.py')):
                # 尝试读取插件信息
                try:
                    # 首先尝试从 main.py 中读取 config 变量
                    main_py_path = os.path.join(plugin_path, 'main.py')
                    plugin_info = {
                        "name": plugin_name,
                        "describe": "",
                        "author": "未知"
                    }
                    
                    # 尝试解析 main.py 中的 config 变量
                    with open(main_py_path, 'r', encoding='utf-8') as f:
                        main_content = f.read()
                        
                        # 使用正则表达式查找 config 变量
                        import re
                        config_match = re.search(r'config\s*=\s*{([^}]*)}', main_content, re.DOTALL)
                        if config_match:
                            config_str = config_match.group(1)
                            
                            # 提取 name
                            name_match = re.search(r"'name'\s*:\s*'([^']*)'", config_str)
                            if name_match:
                                plugin_info["name"] = name_match.group(1)
                            
                            # 提取 describe
                            describe_match = re.search(r"'describe'\s*:\s*'([^']*)'", config_str)
                            if describe_match:
                                plugin_info["describe"] = describe_match.group(1)
                            
                            # 提取 author
                            author_match = re.search(r"'author'\s*:\s*'([^']*)'", config_str)
                            if author_match:
                                plugin_info["author"] = author_match.group(1)
                    
                    # 如果仍然没有描述，尝试从其他位置获取
                    if not plugin_info["describe"]:
                        # 尝试从 info.yaml 读取
                        info_path = os.path.join(plugin_path, 'info.yaml')
                        if os.path.exists(info_path):
                            with open(info_path, 'r', encoding='utf-8') as f:
                                import yaml
                                info = yaml.safe_load(f)
                                if info:
                                    if "describe" in info:
                                        plugin_info["describe"] = info["describe"]
                    
                    # 如果仍然没有描述，使用默认描述
                    if not plugin_info["describe"]:
                        plugin_info["describe"] = f"{plugin_name} 插件"
                    
                    preloaded_plugins.append(plugin_info)
                except Exception as e:
                    print(f"预加载插件 {plugin_name} 信息时出错: {str(e)}")
                    # 即使出错，也添加这个插件
                    preloaded_plugins.append({
                        "name": plugin_name,
                        "describe": f"{plugin_name} 插件",
                        "author": "未知"
                    })
        
        # 按名称排序
        preloaded_plugins.sort(key=lambda x: x["name"])
        
        print(f"预加载了 {len(preloaded_plugins)} 个插件")
        plugins_loaded = True  # 标记插件已加载
    except Exception as e:
        print(f"预加载插件列表时出错: {str(e)}")
        import traceback
        traceback.print_exc()

# 添加刷新插件列表的API
@app.route('/api/refresh_plugins', methods=['POST'])
def refresh_plugins():
    """强制刷新插件列表"""
    global plugins_loaded
    plugins_loaded = False  # 重置加载标志
    preload_plugins()  # 重新加载插件
    return jsonify({"success": True, "message": f"已刷新插件列表，共 {len(preloaded_plugins)} 个插件", "type": "info"})

def get_session_images(session_data):
    """从会话数据中提取图片文件名"""
    image_files = []
    for message in session_data:
        if message.get("type") == "image":
            image_name = message.get("content")
            if image_name:
                image_files.append(image_name)
    return image_files

@app.route('/api/history/<filename>', methods=['GET', 'DELETE'])
def history_session(filename):
    """获取或删除特定历史会话的内容"""
    if request.method == 'GET':
        session_data = load_session_history(filename)
        return jsonify(session_data)
    elif request.method == 'DELETE':
        try:
            # 先加载会话数据以获取图片文件名
            session_data = load_session_history(filename)
            image_files = get_session_images(session_data)
            
            # 删除相关图片文件
            uploads_dir = os.path.join(value.data_path, "uploads")
            for image_file in image_files:
                image_path = os.path.join(uploads_dir, image_file)
                if os.path.exists(image_path):
                    os.remove(image_path)
                    print(f"已删除图片文件: {image_path}")
            
            # 删除历史记录文件
            filepath = os.path.join(HISTORY_DIR, filename)
            if os.path.exists(filepath):
                os.remove(filepath)
                
                # 更新历史记录索引
                for plugin_name, sessions in debug_state["history"].items():
                    debug_state["history"][plugin_name] = [
                        session for session in sessions if session["filename"] != filename
                    ]
                
                # 保存更新后的历史记录索引
                history_index_path = os.path.join(HISTORY_DIR, "history_index.json")
                with open(history_index_path, 'w', encoding='utf-8') as f:
                    json.dump(debug_state["history"], f, ensure_ascii=False, indent=2)
                
                return jsonify({"success": True, "message": "历史记录及相关图片已删除", "type": "info"})
            else:
                return jsonify({"success": False, "message": "历史记录不存在", "type": "warning"}), 404
        except Exception as e:
            return jsonify({"success": False, "message": f"删除历史记录失败: {str(e)}", "type": "error"}), 500

# 修改 get_plugins 路由，使用预加载的插件信息
@app.route('/api/plugins')
def get_plugins():
    """获取可用插件列表"""
    global preloaded_plugins
    # 确保插件已加载
    if not plugins_loaded:
        preload_plugins()
    return jsonify(preloaded_plugins)

# 修改 start_debug 路由
@app.route('/api/start_debug', methods=['POST'])
def start_debug():
    """启动插件调试"""
    data = request.json
    plugin_name = data.get('plugin_name')
    
    if not plugin_name:
        return jsonify({"success": False, "message": "未指定插件名称"}), 400
    
    # 如果已经有正在调试的插件，先停止它
    if debug_state["active"]:
        save_history()
        try:
            asyncio.run(driver.run_shutdown())
        except Exception as e:
            print(f"停止之前的调试失败: {str(e)}")
    
    debug_state["plugin_name"] = plugin_name
    debug_state["active"] = True
    debug_state["current_session"] = []
    
    # 启动插件
    try:
        asyncio.run(driver.run_startup())
        return jsonify({"success": True, "message": f"开始调试插件: {plugin_name}", "type": "success"})
    except Exception as e:
        debug_state["active"] = False
        return jsonify({"success": False, "message": f"启动插件失败: {str(e)}", "type": "error"}), 500

# 修改 stop_debug 路由
@app.route('/api/stop_debug', methods=['POST'])
def stop_debug():
    """停止插件调试"""
    if debug_state["active"]:
        save_history()
        debug_state["active"] = False
        try:
            asyncio.run(driver.run_shutdown())
            return jsonify({"success": True, "message": "调试已停止，历史记录已保存", "type": "info"})
        except Exception as e:
            return jsonify({"success": False, "message": f"停止调试失败: {str(e)}", "type": "error"}), 500
    else:
        return jsonify({"success": False, "message": "没有正在进行的调试", "type": "warning"}), 400

@app.route('/api/history')
def get_history():
    """获取历史记录列表"""
    load_history()
    return jsonify(debug_state["history"])

@app.route('/api/history/plugin/<plugin_name>', methods=['DELETE'])
def delete_plugin_history(plugin_name):
    """删除特定插件的所有历史记录"""
    try:
        if plugin_name in debug_state["history"]:
            # 删除文件和相关图片
            for session in debug_state["history"][plugin_name]:
                # 加载会话数据以获取图片文件名
                session_data = load_session_history(session["filename"])
                image_files = get_session_images(session_data)
                
                # 删除相关图片文件
                uploads_dir = os.path.join(value.data_path, "uploads")
                for image_file in image_files:
                    image_path = os.path.join(uploads_dir, image_file)
                    if os.path.exists(image_path):
                        os.remove(image_path)
                        print(f"已删除图片文件: {image_path}")
                
                # 删除历史记录文件
                filepath = os.path.join(HISTORY_DIR, session["filename"])
                if os.path.exists(filepath):
                    os.remove(filepath)
            
            # 从历史记录索引中删除
            del debug_state["history"][plugin_name]
            
            # 保存更新后的历史记录索引
            history_index_path = os.path.join(HISTORY_DIR, "history_index.json")
            with open(history_index_path, 'w', encoding='utf-8') as f:
                json.dump(debug_state["history"], f, ensure_ascii=False, indent=2)
            
            return jsonify({"success": True, "message": f"已删除 {plugin_name} 的所有历史记录及相关图片"})
        else:
            return jsonify({"success": False, "message": "插件历史记录不存在"}), 404
    except Exception as e:
        return jsonify({"success": False, "message": f"删除历史记录失败: {str(e)}"}), 500

@app.route('/api/history', methods=['GET', 'DELETE'])
def history():
    """获取或删除所有历史记录"""
    if request.method == 'GET':
        load_history()
        return jsonify(debug_state["history"])
    elif request.method == 'DELETE':
        try:
            # 删除所有历史记录文件及相关图片
            uploads_dir = os.path.join(value.data_path, "uploads")
            for plugin_name, sessions in debug_state["history"].items():
                for session in sessions:
                    # 加载会话数据以获取图片文件名
                    session_data = load_session_history(session["filename"])
                    image_files = get_session_images(session_data)
                    
                    # 删除相关图片文件
                    for image_file in image_files:
                        image_path = os.path.join(uploads_dir, image_file)
                        if os.path.exists(image_path):
                            os.remove(image_path)
                            print(f"已删除图片文件: {image_path}")
                    
                    # 删除历史记录文件
                    filepath = os.path.join(HISTORY_DIR, session["filename"])
                    if os.path.exists(filepath):
                        os.remove(filepath)
            
            # 清空历史记录索引
            debug_state["history"] = {}
            
            # 保存空的历史记录索引
            history_index_path = os.path.join(HISTORY_DIR, "history_index.json")
            with open(history_index_path, 'w', encoding='utf-8') as f:
                json.dump(debug_state["history"], f, ensure_ascii=False, indent=2)
            
            return jsonify({"success": True, "message": "所有历史记录及相关图片已删除"})
        except Exception as e:
            return jsonify({"success": False, "message": f"删除历史记录失败: {str(e)}"}), 500

@app.route('/uploads/<path:filename>')
def uploaded_file(filename):
    """提供上传的文件"""
    return send_from_directory(os.path.join(value.data_path, "uploads"), filename)

# WebSocket事件
@socketio.on('connect')
def handle_connect():
    """处理WebSocket连接"""
    emit('status', {"connected": True, "active": debug_state["active"], "plugin": debug_state["plugin_name"]})

@socketio.on('send_message')
def handle_message(data):
    """处理通过WebSocket发送的消息"""
    if not debug_state["active"]:
        emit('error', {"message": "没有正在进行的调试会话"})
        return
    
    asyncio.run(handle_simulated_message(data))

@socketio.on('upload_file')
def handle_upload(data):
    """处理文件上传"""
    file_data = data.get('file')
    file_type = data.get('type', 'image')
    filename = data.get('filename', f"{int(time.time())}.{file_type}")
    
    upload_dir = os.path.join(value.data_path, "uploads")
    os.makedirs(upload_dir, exist_ok=True)
    
    filepath = os.path.join(upload_dir, filename)
    with open(filepath, 'wb') as f:
        f.write(file_data)
    
    emit('upload_success', {"filename": filename, "path": filepath})

def parse_args():
    """解析命令行参数"""
    parser = argparse.ArgumentParser(description='洛玖机器人调试工具')
    parser.add_argument('plugin_name', nargs='?', help='要调试的插件名称')
    parser.add_argument('--host', default='127.0.0.1', help='调试服务器主机地址')
    parser.add_argument('--port', type=int, default=5000, help='调试服务器端口')
    return parser.parse_args()


args = parse_args()
# 如果指定了插件名称，自动启动调试
if args.plugin_name:
    debug_state["plugin_name"] = args.plugin_name
    debug_state["active"] = True
    asyncio.run(driver.run_startup())
    print(f"开始调试插件: {args.plugin_name}")

# 加载历史记录
load_history()

# 启动Flask服务器
url = f"http://{args.host}:{args.port}"
print(f"调试服务器运行在 {url}")
# webbrowser.open(url)

socketio.run(app, host=args.host, port=args.port, debug=True, allow_unsafe_werkzeug=True)