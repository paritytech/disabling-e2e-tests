# Validator disabling e2e tests

Tests for https://github.com/paritytech/polkadot-sdk/issues/1590.

## Setup

The test requires the following binaries to be available in the $PATH:
* polkadot (compiled with `--features=fast-runtime`)
* polkadot-parachain (cumulus-based collator)
* malus

## Running

```
cargo test -- --nocapture
```

## Current status

Due to zombienet-sdk [limitations](https://github.com/paritytech/zombienet-sdk/pull/145), we use a branch of it with some fixes applied.

As of time of this writing (Dec 2023), zombienet-sdk only supports running
tests with a native provider. This is why the tests are not integrated into
polkadot-sdk CI pipeline yet. Once zombienet-sdk is ready, this repo should be
removed and the tests moved to polkadot-sdk.
