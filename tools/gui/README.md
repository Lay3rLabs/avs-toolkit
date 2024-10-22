# Climb Web Demo

## Prerequisites

* Trunk: https://trunkrs.dev/

## Environment vars

`.env` will be read to add environment vars while building and running

#### Contract Ids

Not required, but helpful, especially for local dev

In the `cli` directory: `cargo run -- --target local upload contracts`

Then set the following vars in your environment

```
LOCAL_CODE_ID_TASK_QUEUE={CODE ID FOR TASK QUEUE}
LOCAL_CODE_ID_MOCK_OPERATORS={CODE ID FOR TASK MOCK OPERATORS}
LOCAL_CODE_ID_VERIFIER_SIMPLE={CODE ID FOR TASK VERIFIER SIMPLE}
LOCAL_CODE_ID_VERIFIER_ORACLE={CODE ID FOR TASK VERIFIER ORACLE}
```

Adjust as needed for testnet

#### Wallet

Also not required, but if developing locally with the autoconnect feature, set `LOCAL_MNEMONIC` and/or `TEST_MNEMONIC`

## Run in browser

In order to not conflict with other services, we'll run it on port 4041

```
trunk serve --port=4041
```

To skip past the initial wallet connect page:

```
trunk serve --features=autoconnect --port=4041
```

And if you're making changes to the shared package, add it to the watcher too

```
trunk serve --features=autoconnect --port=4041 --watch . --watch ../shared
```
