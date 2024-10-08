# Simple AVS Squaring example in Python

This builds a simple WASI component that squares whatever number is input into
it, which can be compared with
[Eigenlayer's Incredible Squaring AVS Example](https://github.com/Layr-Labs/incredible-squaring-avs)
to demonstate how much less boilerplate is needed for building on Layer.

## Setup

First, install [Python 3.10 or later](https://www.python.org/) and
[pip](https://pypi.org/project/pip/) if you don't already have them.  Then,
install [`componentize-py`](https://github.com/bytecodealliance/componentize-py):

```bash
pip install componentize-py
```

Optional, but recommended, install [`jco`](https://github.com/bytecodealliance/jco) to easily
optimize the binary for size before deploying.

## Build

In this current working directory, build a component from the logic in the `app.py` file.
We provide the path to the `wit` directory that has the WIT IDL files that describe the
Wasm component that we want to create.

```bash
componentize-py -d ../../wit -w task-queue componentize --stub-wasi app -o app.wasm
```

The outputted `app.wasm` is deployable as is, but it is `34 MB` in size. If we run an optimization
step using `jco`, we can get that file size down to `14.5 MB`.

```bash
jco opt app.wasm -o app.wasm -- --strip-debug -Oz --enable-bulk-memory
```

## Deploy

Upload the compiled Wasm component to the Wasmatic node using the `avs-toolkit-cli` CLI tool (if you don't have it already,
`cargo install --path ../../tools/cli`).
Assign a unique name, as it is how your application is going to be distinguished. The examples below assume
the assigned name is `pysquare`.

You'll also need to use the task address that was created when you deployed your contract.

```bash
avs-toolkit-cli wasmatic deploy --name pysquare \
    --wasm-source ./app.wasm  \
    --testable \
    --task <TASK-ADDRESS>
```

## Testing Deployment

This can only be done if `--testable` flag was provided during deployment.
To test the deployed application on the Wasmatic node, you can provide `input` test data
that your application expects. The server responds with the output of the applicaton without
sending the result to the chain. For the input flag, you'll use the json input expected
by the WASI component you're testing.

```bash
avs-toolkit-cli wasmatic test --name pysquare --input '{"x":9}'
```

## Test Executions

You can use `avs-toolkit-cli` to test your component locally before you deploy, as well as after.

To test and execute locally, run the following with parameters for the expected input JSON for the app:

```bash
avs-toolkit-cli wasmatic run \
    --wasm-source ./app.wasm  \
    --input '{"x":9}'
```

You can also test your component remotely after you've deployed it. This is only possible if you used the `--testable` flag during deployment.

The server responds with the output of the applicaton without sending the result to the chain. For the input flag, you'll use the json input expected by the WASI component you're testing.

```bash
avs-toolkit-cli wasmatic test --name pysquare --input '{"x":9}'
```

It will parse the input as if you pushed it to the task queue and return the result (or error) to the caller.

## Creating New Python Services

If you'd like to start a new Python service from scratch, there's just a couple extra steps.
First, let's install [`wkg`](https://github.com/bytecodealliance/wasm-pkg-tools) CLI if you haven't already.
This requires Rust 1.80+. Please ensure you have that installed via `rustup` before continuing.

Install `wkg` CLI:
```bash
cargo install wkg
```

Set default registry configuration:
```bash
wkg config --default-registry wa.dev
```
For more information about configuration, see
the [wkg docs](https://github.com/bytecodealliance/wasm-pkg-tools).

Now, let's go to new directory where you would like to develop your service. Start by downloading the
WIT interfaces that we will need to build a task queue triggered application.

```bash
mkdir wit && wkg get lay3r:avs -o wit/ && wkg wit fetch
```
This creates a new `wit` subdirectory and downloads the WIT files needed for creating a task queue app.

```bash
componentize-py -d ./wit -w task-queue bindings <new-python-package-name> && cd <new-python-package-name>
```

Create a new file `app.py` with something like our square example:

```python
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
```

Now you can build the component:

```bash
componentize-py -d ../wit -w task-queue componentize --stub-wasi app -o app.wasm
```

And optional optimize the binary size:
```bash
jco opt app.wasm -o app.wasm -- --strip-debug -Oz --enable-bulk-memory
```
