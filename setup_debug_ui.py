import os
import shutil

# 定义目录结构
directories = [
    'debug_ui/templates',
    'debug_ui/static/css',
    'debug_ui/static/js',
    'debug_ui/static/img'
]

# 创建目录
for directory in directories:
    os.makedirs(directory, exist_ok=True)
    print(f"创建目录: {directory}")

# 创建示例提示文件
with open('debug_ui/README.txt', 'w', encoding='utf-8') as f:
    f.write("""
洛玖机器人调试界面
=================

使用方法:
1. 运行 python debug_launcher.py [插件名称]
2. 在浏览器中访问 http://127.0.0.1:5000

功能:
- 可视化调试插件
- 支持发送文本、图片、音频和视频
- 保存调试历史记录
- 查看历史调试会话
    """)

print("调试界面设置完成！")
print("运行 'python setup_debug_ui.py' 创建必要的目录结构")
print("运行 'python debug_launcher.py [插件名称]' 启动调试界面")