from typing import Callable, Coroutine, Any, List, Dict
from apscheduler.schedulers.asyncio import AsyncIOScheduler
import asyncio

class Task:
    def __init__(self):
        self._scheduler = AsyncIOScheduler()
        self._tasks: List[Dict] = []
        self._job_mapping: Dict[Callable, str] = {}
        
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
            job = self._scheduler.add_job(
                task['func'],
                trigger=task['trigger'],
                **task['kwargs']
            )
            self._job_mapping[task['func']] = job.id

        self._scheduler.start()
        try:
            await asyncio.Event().wait()
        except (KeyboardInterrupt, SystemExit):
            self._scheduler.shutdown()

    def adjust_interval(self, func: Callable, trigger: str, **kwargs):
        """动态调整任务间隔"""
        if func not in self._job_mapping:
            raise ValueError("任务未注册")
        
        job_id = self._job_mapping[func]
        self._scheduler.reschedule_job(
            job_id,
            trigger=trigger,
            **kwargs
        )
        print(f"已调整 {func.__name__} 新定时任务 {trigger} : {kwargs}")

    def add_task(self, func: Callable[[], Coroutine[Any, Any, None]], trigger: str, **kwargs):
        """动态添加新的定时任务"""
        print(f"添加新定时任务：{func.__name__}\t{trigger}\t{kwargs}")
        job = self._scheduler.add_job(
            func,
            trigger=trigger,
            **kwargs
        )
        self._job_mapping[func] = job.id
        print(f"新定时任务 {func.__name__} 已添加")

task = Task()