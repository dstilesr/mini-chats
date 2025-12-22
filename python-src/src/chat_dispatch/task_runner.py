import asyncio
import logging
from dataclasses import dataclass, field
from multiprocessing import Lock
from multiprocessing.synchronize import Lock as SLock
from typing import Coroutine

from .singleton import Singleton

logger = logging.getLogger(__name__)


@dataclass(slots=True)
class TaskRunner(metaclass=Singleton):
    """
    Class to run coroutines in the background.
    """

    tasks: set[asyncio.Task] = field(default_factory=set)
    lock: SLock = field(default_factory=Lock)

    def dispatch_task(self, coro: Coroutine[None, None, None]) -> str:
        """
        Dispatches a coroutine to run in the background as a Task. Returns
        the task name.
        """
        task = asyncio.create_task(coro)
        with self.lock:
            task.add_done_callback(self._remove_task)
            self.tasks.add(task)

        logger.debug("Dispatched task %s", task.get_name())
        return task.get_name()

    def _remove_task(self, task: asyncio.Task):
        """
        Remove task from set after completion for cleanup.
        """
        with self.lock:
            self.tasks.remove(task)

        logger.debug("Task %s exited", task.get_name())

    def stop_task(self, task_id: str):
        """
        Stops a task by its ID.
        """
        with self.lock:
            task = next(
                (t for t in self.tasks if t.get_name() == task_id), None
            )
            if task:
                task.cancel()

        logger.debug("Stopped task %s", task_id)


runner = TaskRunner()
