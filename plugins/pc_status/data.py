import json
import base64
from datetime import datetime
from io import BytesIO
import matplotlib
import matplotlib.pyplot as plt
import matplotlib.dates as mdates

# 添加下载的字体文件
matplotlib.font_manager.fontManager.addfont('chinese.simhei.ttf')
# 设置 Matplotlib 使用 SimHei 字体
matplotlib.rc('font', family='SimHei')

def analyze_disk_events(json_data):
    """解析磁盘事件JSON数据，返回增强的分析结果"""
    events = json.loads(json_data)
    results = []

    for event in events:
        # 基础信息提取
        start_dt = datetime.fromtimestamp(event["start"])
        end_dt = datetime.fromtimestamp(event["end"])

        event_info = {
            "operation": event["type"],
            "start_time": start_dt.strftime('%Y-%m-%d %H:%M:%S'),
            "start_timestamp": event["start"],
            "end_time": end_dt.strftime('%Y-%m-%d %H:%M:%S'),
            "end_timestamp": event["end"],
            "duration_sec": round(event["duration"], 2),
            "max_value_bytes": event["max_value"],
            "max_value_human": _bytes_to_human(event["max_value"]),
            "peak_timestamp": None,
            "peak_time_human": None,
            "data_points": event["data"]
        }

        # 查找峰值时间
        for dp in event["data"]:
            if dp["value"] == event["max_value"]:
                peak_dt = datetime.fromtimestamp(dp["timestamp"])
                event_info["peak_timestamp"] = dp["timestamp"]
                event_info["peak_time_human"] = peak_dt.strftime('%Y-%m-%d %H:%M:%S.%f')[:-3]
                break

        results.append(event_info)

    return results


def _bytes_to_human(size):
    """智能字节转换"""
    units = ['B', 'KB', 'MB', 'GB']
    for unit in units:
        if size < 1024.0 or unit == 'GB':
            break
        size /= 1024.0
    return f"{size:.2f} {unit}"


def generate_html_report(analysis_results, output_file="report.html"):
    """生成包含可视化图表的HTML报告"""
    html = f"""
    <html>
    <head>
        <meta http-equiv="Content-Type" content="text/html;charset=utf-8"/>
        <title>磁盘写入事件分析报告</title>
        <style>
            body {{ font-family: Arial, sans-serif; margin: 2em; }}
            .event {{ border: 1px solid #ddd; padding: 1.5em; margin-bottom: 2em; border-radius: 8px; }}
            table {{ width: 100%; border-collapse: collapse; margin: 1em 0; }}
            th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
            th {{ background-color: #f8f9fa; }}
            img {{ max-width: 100%; height: auto; border: 1px solid #eee; }}
            .plot-title {{ color: #2c3e50; margin-top: 1.5em; }}
        </style>
    </head>
    <body>
        <h1>磁盘写入事件分析报告</h1>
        <p>生成时间：{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
    """

    for idx, event in enumerate(analysis_results, 1):
        # 生成图表
        img_base64 = _generate_plot(event)

        # 构建事件信息表格
        html += f"""
        <div class="event">
            <h2>事件 #{idx}</h2>
            <table>
                <tr><th>操作类型</th><td>{event['operation'].upper()}</td></tr>
                <tr><th>开始时间</th><td>{event['start_time']}</td></tr>
                <tr><th>结束时间</th><td>{event['end_time']}</td></tr>
                <tr><th>持续时间</th><td>{event['duration_sec']} 秒</td></tr>
                <tr><th>峰值大小</th><td>{event['max_value_human']} ({event['max_value_bytes']} 字节)</td></tr>
                <tr><th>峰值时间</th><td>{event['peak_time_human']}</td></tr>
            </table>

            <h3 class="plot-title">写入量时序图</h3>
            <img src="data:image/png;base64,{img_base64}">
        </div>
        """

    html += "</body></html>"

    with open(output_file, "w", encoding='utf-8') as f:
        f.write(html)
    print(f"报告已生成：{output_file}")


def _generate_plot(event):
    """生成时序图并返回base64编码"""
    # 准备数据
    timestamps = [dp["timestamp"] for dp in event["data_points"]]
    values = [dp["value"] for dp in event["data_points"]]
    peak_value = event["max_value_bytes"]

    # 转换为datetime对象
    dates = [datetime.fromtimestamp(ts) for ts in timestamps]

    # 创建图表
    plt.figure(figsize=(12, 6))
    ax = plt.gca()

    # 绘制主曲线
    line, = ax.plot(dates, values, 'b-', marker='o', markersize=5,
                    linewidth=1.5, markerfacecolor='#e74c3c', markeredgewidth=1)

    # 标记峰值点
    peak_index = values.index(peak_value)
    ax.plot(dates[peak_index], peak_value, 'ro', markersize=8,
            markerfacecolor='none', markeredgewidth=2)
    ax.annotate(f'峰值: {_bytes_to_human(peak_value)}',
                xy=(dates[peak_index], peak_value),
                xytext=(15, 15), textcoords='offset points',
                arrowprops=dict(arrowstyle="->", color='#e74c3c'))

    # 格式化坐标轴
    ax.xaxis.set_major_formatter(mdates.DateFormatter('%H:%M:%S'))
    ax.xaxis.set_major_locator(mdates.SecondLocator(interval=2))
    plt.xticks(rotation=45)
    plt.ylabel("写入量 (字节)")
    plt.title(f"磁盘写入时序图 - {event['operation'].upper()}")
    plt.grid(True, linestyle='--', alpha=0.7)

    # 保存为base64
    buf = BytesIO()
    plt.savefig(buf, format='png', bbox_inches='tight')
    plt.close()
    return base64.b64encode(buf.getvalue()).decode('utf-8')


# 使用示例
if __name__ == "__main__":
    # 从文件读取数据（替换为实际路径）
    with open("disk_events2.json", "r") as f:
        json_data = f.read()

    # 分析数据并生成报告
    analysis = analyze_disk_events(json_data)
    generate_html_report(analysis)