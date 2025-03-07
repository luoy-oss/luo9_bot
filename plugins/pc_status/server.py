import psutil
import time
from collections import deque

__all__ = ["check_alert_conditions", 
           "get_cpu_status", "get_disk_read_status", "get_disk_write_status", "get_memory_status", 
           "get_system_info", "get_current_status"]

__cpu = "cpu"
__memory = "memory"
__disk_write = "disk_write"
__disk_read = "disk_read"

# 配置项
CONFIG = {
    "refresh_interval": 1,
    "thresholds": {
        f"{__cpu}":         95,  # CPU使用率阈值%
        f"{__memory}":      90,  # 内存使用率阈值%
        f"{__disk_write}":  300 * 1024 * 1024,  # 磁盘写入速率阈值
        f"{__disk_read}":   300 * 1024 * 1024,  # 磁盘读入速率阈值
    },
    "alert_cooldown": 300,  # 报警冷却时间(秒)
    "history": {
        "max_points": 3600,  # 保留1小时数据（每秒1个点）
        "chart_points": 300   # 图表显示最近5分钟数据
    },
    "report": {
        "storage_days": 7     # 报告保留天数
    }
}

# 全局状态
history_data = {
    f"{__cpu}":         deque(maxlen=CONFIG['history']['max_points']),
    f"{__memory}":      deque(maxlen=CONFIG['history']['max_points']),
    f"{__disk_write}":  deque(maxlen=CONFIG['history']['max_points']),
    f"{__disk_read}":   deque(maxlen=CONFIG['history']['max_points'])
}



class AlertTracker:
    def __init__(self, metric_type):
        self.metric_type = metric_type
        self.start_time = None
        self.max_value = 0
        self.data_points = []
        self.active = False

    def add_point(self, value, timestamp):
        if not self.active:
            self.active = True
            self.start_time = timestamp
            self.max_value = value
            self.data_points = []

        if value > self.max_value:
            self.max_value = value

        self.data_points.append({
            "timestamp": timestamp,
            "value": value
        })

    def finalize(self, end_time):
        duration = end_time - self.start_time
        report = {
            "type": self.metric_type,
            "start": self.start_time,
            "end": end_time,
            "duration": duration,
            "max_value": self.max_value,
            "data": self.data_points[-CONFIG['history']['chart_points']:]
        }
        self.reset()
        return report

    def reset(self):
        self.active = False
        self.start_time = None
        self.max_value = 0
        self.data_points = []

def get_cpu_status(response):
    return response[f"{__cpu.upper()}"]['status']

def get_memory_status(response):
    return response[f"{__memory.upper()}"]['status']

def get_disk_write_status(response):
    return response[f"{__disk_write.upper()}"]['status']

def get_disk_read_status(response):
    return response[f"{__disk_read.upper()}"]['status']

def check_alert_conditions(data):
    response = {}
    current_time = time.time()

    # 存储历史数据
    history_data[f'{__cpu}'].append((current_time, data['cpu']['total']))
    history_data[f'{__memory}'].append((current_time, data['memory']['percent']))
    history_data[f'{__disk_write}'].append((current_time, data['disk']['write_rate']))
    history_data[f'{__disk_read}'].append((current_time, data['disk']['read_rate']))

    metrics = {
        __cpu: data['cpu']['total'],
        __memory: data['memory']['percent'],
        __disk_write: data['disk']['write_rate'],
        __disk_read: data['disk']['read_rate']
    }

    for metric, value in metrics.items():
        status = "alert" if value > CONFIG['thresholds'][metric] else "ok"
        response[metric.upper()] = {"status": status}

    return response


def bytes_to_human(b):
    units = ['B', 'KB', 'MB', 'GB', 'TB']
    i = 0
    while b >= 1024 and i < len(units) - 1:
        b /= 1024
        i += 1
    return f"{b:.2f} {units[i]}"


def get_system_info(prev_disk: psutil._common.sdiskio):
    cpu = psutil.cpu_times_percent(interval=1)
    mem = psutil.virtual_memory()
    net = psutil.net_io_counters()
    disk = psutil.disk_io_counters()

    disk_write = disk.write_bytes - prev_disk.write_bytes if prev_disk else 0
    disk_read = disk.read_bytes - prev_disk.read_bytes if prev_disk else 0

    return {
        "cpu": {
            "user": cpu.user,
            "sys": cpu.system,
            "steal": getattr(cpu, 'steal', 0),
            "total": 100 - cpu.idle
        },
        "memory": {
            "used": mem.used,
            "total": mem.total,
            "percent": mem.percent
        },
        "network": {
            "sent": net.bytes_sent,
            "recv": net.bytes_recv
        },
        "disk": {
            "write_rate": disk_write,
            "read_rate": disk_read
        }
    }, disk


def get_current_status(pre_disk):
    data, _ = get_system_info(pre_disk)
    data['disk']['write_rate'] = bytes_to_human(data['disk']['write_rate'])
    data['disk']['read_rate'] = bytes_to_human(data['disk']['read_rate'])
    return data