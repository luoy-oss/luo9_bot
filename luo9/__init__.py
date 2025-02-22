__all__ = ['tasks', 'drivers', 'message', 'notice']

from .drivers import driver
from .tasks import task

def get_driver():
    if driver is not None:
        return driver
    raise ValueError("Driver 对象未初始化")

def get_task():
    if task is not None:
        return task
    raise ValueError("Task 对象未初始化")