import task_queue
from task_queue import TaskQueue
from task_queue.types import Ok, Result
from task_queue.imports.lay3r_avs_types import TaskQueueInput

import json

class TaskQueue(TaskQueue):
    def run_task(self, request: TaskQueueInput) -> Result[bytes, str]:
        # parse the json input
        x = json.loads(request.request)["x"]

        # square the number
        y = x * x

        # serialize the json output as bytes
        output = json.dumps({"y": y}).encode('utf-8')
        
        # return output
        return Ok(output)
