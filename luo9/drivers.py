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
    
    async def run_startup(self):
        for callback in self._startup_callbacks:
            await callback()

    async def run_shutdown(self):
            for callback in self._shutdown_callbacks:
                await callback()

driver = Driver()
