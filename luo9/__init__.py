__all__ = ['drivers', 'message', 'notice']

from .drivers import driver

def get_driver():
    if driver is not None:
        return driver
    raise ValueError("Driver 对象未初始化")