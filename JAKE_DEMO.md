# Containerized AVSs Demo for Eigen

Show frontend. Square 3, the answer is 9.


Make change. Compile.
```
cargo component build --release
```


Deploy new service contracts (pre-build no changes made).
```
avs-toolkit-cli deploy contracts --artifacts-path ../../artifacts --operators wasmatic --title="Cubed" --description="Another uponly math service"
```

Add task queue address environment variable.

Get the operators.
```
avs-toolkit-cli task-queue view-queue
```


Copy address, paste in frontend. 

Deploy new wasm.
```
avs-toolkit-cli wasmatic deploy --name better-math \
    --wasm-source ./target/wasm32-wasip1/release/cavs_squared.wasm  \
    --testable \
    --task $TEST_TASK_QUEUE_ADDRESS

```


Go to frontend and make a task. 3 is now 27. We deployed a new AVS.
