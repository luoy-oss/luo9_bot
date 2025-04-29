from typing import Callable, Coroutine, Any, List

class Driver:
    def __init__(self):
        self._startup_callbacks: List[Callable[[], Coroutine[Any, Any, None]]] = []
        self._shutdown_callbacks: List[Callable[[], Coroutine[Any, Any, None]]] = []

    def on_startup(self, func: Callable[[], Coroutine[Any, Any, None]]):
        self._startup_callbacks.append(func)
        return func
    
    def on_shutdown(self, func: Callable[[], Coroutine[Any, Any, None]]):
        self._shutdown_callbacks.append(func)
        return func
    
    def run_startup(self):
        import asyncio
        for callback in self._startup_callbacks:
            asyncio.run(callback())

    def run_shutdown(self):
        import asyncio
        for callback in self._shutdown_callbacks:
            asyncio.run(callback())

driver = Driver()
