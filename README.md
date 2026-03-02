# Polkadot SDK / XCM

This repository contains a parachain runtime and a small pallet that builds XCM programs for Asset Hub (Westend) interoperability.

**Location Hierarchy**

Locations used in this project, from the local parachain perspective:
- `Here`: the local parachain (para id from `ParachainInfo`).
- `Parent`: the relay chain (Westend).
- `Parent -> Parachain(1000)`: Asset Hub (Westend).
- `AccountId32`: accounts, used as beneficiaries or refund targets.

**XCM Programs**

1. `register_native_asset_on_ah` steps:
1. Origin is `Root`.
1. Build the native asset location from Asset Hub’s perspective: `Parent -> Parachain(2000)`.
1. Encode `ForeignAssets::create` with `id` = parachain location, `admin` = sovereign account, `min_balance` = `1`.
1. Pay execution with WND (parent asset), intentionally overestimating.
1. Use `RefundSurplus` and `DepositAsset` to return leftover fees to the sovereign account on Asset Hub.
1. Send to `Parent -> Parachain(1000)`.

2. `transfer_to_ah` steps:
1. Origin is signed.
1. Execute an XCM program locally (so `InitiateTransfer` is processed by the executor).
1. Withdraw WND (plus extra WND for remote fees), then withdraw native.
1. Use `InitiateTransfer` to send to Asset Hub:
1. Pay remote fees in WND using `ReserveWithdraw` (Asset Hub is configured as reserve for WND in this runtime).
1. Teleport the native asset.
1. Reserve-withdraw WND to Asset Hub.
1. Deposit the two transferred assets to the beneficiary.
1. Refund any surplus remote fees and deposit the refund to the sovereign account on Asset Hub.

**Runtime XCM Configuration Changes**

1. `IsTeleporter` is set to `NativeAssetToAssetHub`.
1. Teleports are restricted to the local native asset and only to Asset Hub.
1. This keeps teleports tightly whitelisted and aligned with best practices.

**Tests and Logs**

Run integration tests:
```bash
cargo test -p emulated-tests
```

Enable XCM logs:
```bash
RUST_LOG=xcm cargo test -p emulated-tests
```

**Tradeoffs / Best Practices**

- Reserve transfers are generally safer than teleports. This repo keeps teleports restricted to a strict whitelist (`NativeAssetToAssetHub`), and relies on WND reserve behavior for fees.
- Overestimated fees are refunded via `RefundSurplus + DepositAsset` to avoid trapping assets.
- In the emulator, WND is configured as a non-sufficient asset. Holding any WND adds a consumer ref, so fully withdrawing the native balance would fail unless the account keeps the minimum balance. The pallet temporarily tops up the minimum balance to preserve the account provider ref while transferring the full native amount.

**Additional: Pool / Pay Fees With Parachain Asset**

- Added `setup_pool_on_ah` to create a liquidity pool on Asset Hub between WND and the parachain asset and add initial liquidity.
- Once the pool exists, Asset Hub can swap the parachain asset to WND for XCM fees via the asset conversion trader.

**Additional: Barrier (Banned Account)**

- Added a barrier rule that rejects XCM whose origin is `Parent -> Parachain(1000) -> AccountId32(banned)`.
- The banned account is configured in `runtime/src/configs/xcm_config.rs` as `BannedAssetHubAccountId`.

**Additional: Zombienet**

Prereqs:
- `polkadot` binary in `PATH`
- `polkadot-omni-node` binary in `PATH`
- `zombienet` installed

Generate the parachain chain spec used by `zombienet-omni-node.toml`:
```bash
polkadot-omni-node chain-spec-builder --chain-spec-path dev_chain_spec.json create --runtime target/debug/wbuild/asset-hub-westend-runtime/asset_hub_westend_runtime.wasm --chain-name asset-hub-local --chain-id asset-hub-local -t local -p 1000 --relay-chain rococo-local default
```

Run the local network:
```bash
zombienet spawn --provider native zombienet-omni-node.toml
```

What to validate:
- Relay chain WS: `ws://127.0.0.1:9944` and `ws://127.0.0.1:9955`
- Parachain (Asset Hub) WS: `ws://127.0.0.1:9988`

Note:
- Zombienet is documented and prepared, but not verified end-to-end in this environment due to snap permission issues when running the `polkadot` binary.

