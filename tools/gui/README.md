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
LOCAL_CODE_ID_TASK_QUEUE=5
LOCAL_CODE_ID_MOCK_OPERATORS=2
LOCAL_CODE_ID_VERIFIER_SIMPLE=4
LOCAL_CODE_ID_VERIFIER_ORACLE=3
```

Adjust as needed for testnet

#### Wallet

Also not required, but if developing locally with the autoconnect feature, set `LOCAL_MNEMONIC` and/or `TEST_MNEMONIC`

## Run in browser

```
trunk serve
```

To skip past the initial wallet connect page:

```
trunk serve --features=autoconnect
```

And if you're making changes to the shared package, add it to the watcher too

```
trunk serve --features=autoconnect --watch . --watch ../shared
```
