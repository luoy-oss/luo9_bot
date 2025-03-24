import os
import sys
import argparse
import subprocess
import time

def parse_args():
    parser = argparse.ArgumentParser(description='洛玖机器人调试启动器')
    parser.add_argument('plugin_name', nargs='?', help='要调试的插件名称')
    parser.add_argument('--host', default='127.0.0.1', help='调试服务器主机地址')
    parser.add_argument('--port', type=int, default=5000, help='调试服务器端口')
    parser.add_argument('--no-browser', action='store_true', help='不自动打开浏览器')
    return parser.parse_args()

def main():
    args = parse_args()
    
    # 构建命令
    cmd = [sys.executable, 'debug.py']
    # 不再默认传递插件名称参数
    if args.host:
        cmd.extend(['--host', args.host])
    if args.port:
        cmd.extend(['--port', str(args.port)])
    
    # 启动调试服务器
    print(f"启动调试服务器: {' '.join(cmd)}")
    process = subprocess.Popen(cmd)
    
    # 等待服务器启动
    time.sleep(2)
    
    try:
        # 等待进程结束
        process.wait()
    except KeyboardInterrupt:
        print("正在关闭调试服务器...")
        process.terminate()
        process.wait()

if __name__ == '__main__':
    main()