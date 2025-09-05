# Assignment

This assignment is designed to evaluate your understanding the most common XCM concepts like how to build programs and configure a runtime.

## Instructions

This repository contains a parachain runtime with some missing or erroneous XCM configurations, and a pallet with calls that need to be implemented.
You will find these in the `runtime` and `pallets` folders, respectively.

The assignment has two parts.
In both parts you'll need to implement pallet calls that use XCM.
Additionally, in the second part you'll need to also modify the runtime's XCM configuration.
The first part doesn't need tweaking the configuration.

You'll use the XCM emulator for testing, you can find the setup in `integration-tests/emulated`.
Remember you can get the XCM logs by running with the environment variable `RUST_LOG=xcm`.

### 1. Register native asset on Asset Hub

For this part you'll implement `register_native_asset_on_ah`.
It must construct an XCM program that registers the chain's native asset on the Asset Hub as a foreign asset and then send it using the provided pallet config associated type `Xcm`.
Only root can call this.
Once the chain's native asset is registered on Asset Hub, it can be sent to and from via **teleports**.

There's a tutorial of how to do it through the UI in [the docs](https://docs.polkadot.com/tutorials/polkadot-sdk/system-chains/asset-hub/register-foreign-asset/).
Be sure to also check [Asset Hub Westend's configuration](https://github.com/paritytech/polkadot-sdk/blob/master/cumulus/parachains/runtimes/assets/asset-hub-westend/src/lib.rs).

### 2. Send native parachain asset and WND to Asset Hub

For this part, you'll implement `transfer_to_ah`.
This call must construct an XCM program that sends both WND and the parachain's native asset to Asset Hub.
It can be called by any signed origin.
You'll need to give the `IsTeleporter` configuration item a value, since it'll be `()` by default.
Overestimating fees is not a problem.

## Testing

It's important to test both your extrinsics and configuration.
To do this, we provide the [emulated-tests](./integration-tests/emulated/Cargo.toml) package with all the chains in this scenario.
For more information, go to [the testing docs](https://docs.polkadot.com/develop/interoperability/test-and-debug/).

## README

Graders will start by reviewing your update [README.md](./README.md) that should describe:
- The location hierarchy of the chains involved.
- How you designed your XCM programs and why.
- How you designed the configuration and why.
- Any compromises you made, and things you would improve if you had time.
- How to run the project.

## Optional tasks

If you have more time after making sure your project has all the functionality, a great README and great tests, there are some additional tasks you could do:
- Refund the overestimated fee so that it's not trapped.
- Create a pool on Asset Hub with the parachain's native asset and use it to pay fees only in the parachain asset.
- Configure the `Barrier` to reject transfers from a particular account on Asset Hub.
- Run the whole network locally with zombienet

## Grading

- Implementation
  - Correctness and accuracy
  - Evidence of using various techniques used in class
  - As close to production ready as possible
- Code Quality
  - Tests and code coverage
  - Use of best practices and efficient code
  - Well documented, with considerations and compromises noted
- Bonus Points
  - Completing Optional tasks, mentioned above
  - Incontestably going above and beyond the requirements above
