# Climb Web Demo

## Prerequisites

* Trunk: https://trunkrs.dev/

## Run in browser

```
trunk serve
```

For quicker development, you can autoconnect a wallet with a mnemonic from `.env` (set in `LOCAL_MNEMONIC`)

```
trunk serve --features=autoconnect
```

And if you're making changes to the climb package, add it to the watcher too

```
trunk serve --features=autoconnect --watch . --watch ../../packages/climb
```
