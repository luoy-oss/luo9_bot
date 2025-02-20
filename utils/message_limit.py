import time

class MessageLimit:
    def __init__(self, name):
        self.name = name
        self.current_time = 0
        self.last_triggered = 0  # 初始化最后一次触发时间为0

    def check(self, interval):
        self.current_time = time.time()
        if self.current_time - self.last_triggered >= interval:
            return True
        return False
    def handle(self):
        # 更新最后触发时间
        self.last_triggered = self.current_time
