from typing import TypeVar, Generic, Union, Optional, Protocol, Tuple, List, Any, Self
from enum import Flag, Enum, auto
from dataclasses import dataclass
from abc import abstractmethod
import weakref

from .types import Result, Ok, Err, Some
from .imports import lay3r_avs_types


class TaskQueue(Protocol):

    @abstractmethod
    def run_task(self, request: lay3r_avs_types.TaskQueueInput) -> Result[bytes, str]:
        raise NotImplementedError

