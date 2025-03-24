import threading
import asyncio


class Timeout:
    def __init__(self, wait, on_timeout):
        self.wait = wait
        self.on_timeout = on_timeout
        self.timer = None

    def __call__(self, fn):
        def wrapped(*args, **kwargs):
            # 执行原函数
            result = fn(*args, **kwargs)
            # 取消之前的定时器
            if self.timer is not None:
                self.timer.cancel()
            # 创建新的定时器
            self.timer = threading.Timer(self.wait, self._on_timer)
            self.timer.start()
            return result
        return wrapped

    def _on_timer(self):
        asyncio.run(self.on_timeout())
