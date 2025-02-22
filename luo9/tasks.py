from typing import Callable, Coroutine, Any, List, Dict
from apscheduler.schedulers.asyncio import AsyncIOScheduler
import asyncio

class Task:
    def __init__(self):
        self._scheduler = AsyncIOScheduler()
        self._tasks: List[Dict] = []

    def on_schedule_task(self, trigger: str, **kwargs):
        def decorator(func: Callable[[], Coroutine[Any, Any, None]]):
            print(f"定时任务创建：{func.__name__}\t{trigger}\t{kwargs}")
            self._tasks.append({
                'func': func,
                'trigger': trigger,
                'kwargs': kwargs
            })
            return func
        return decorator

    async def start(self):
        for task in self._tasks:
            self._scheduler.add_job(
                task['func'],
                trigger=task['trigger'],
                **task['kwargs']
            )
        self._scheduler.start()
        try:
            print(self._tasks)
            print("定时任务执行！")
            await asyncio.Event().wait()  # 等待事件，防止协程退出
        except (KeyboardInterrupt, SystemExit):
            self._scheduler.shutdown()

task = Task()