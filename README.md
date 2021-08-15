# Near_Contract_Rust-DiD
The near DID contract implemented by rust

# Building

To build run:

```shell
env 'RUSTFLAGS=-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
```

# Using this contract

This smart contract will get deployed to your NEAR account. For this example, please create a new NEAR account. Because NEAR allows the ability to upgrade contracts on the same account, initialization functions must be cleared. If you'd like to run this example on a NEAR account that has had prior contracts deployed, please use the `near-cli` command `near delete`, and then recreate it in Wallet. To create (or recreate) an account, please follow the directions on [NEAR Wallet](https://wallet.near.org/).

Switch to `mainnet`. You can skip this step to use `testnet` as a default network.

```shell
export NEAR_ENV=mainnet
```

In the project root, log in to your newly created account with `near-cli` by following the instructions after this command:

```shell
near login
```

To make this tutorial easier to copy/paste, we're going to set an environment variable for your account id. In the below command, replace `MY_ACCOUNT_NAME` with the account name you just logged in with, including the `.near`:

```shell
ID=MY_ACCOUNT_NAME
```

You can tell if the environment variable is set correctly if your command line prints the account name after this command:

```shell
echo $ID
```

Create account:

```shell
near create-account did.$ID --masterAccount $ID --initialBalance 20
```

## Example

Now we can deploy the compiled contract in this example to your account:

```shell
near deploy --wasmFile target/wasm32-unknown-unknown/release/did.wasm  --accountId did.$ID
```

Register a DID identity:

```shell
near call did.$ID  reg_did_using_account --accountId $ID
```

Check the DID identity. (Did:NEAR: Your NEAR account):

```shell
near call did.$ID  get_document  '{"did":"did:near:$ID"}' --accountId $ID
```

Add the controller:

```shell
near call did.$ID  add_controller  '{"controller":"did:near:$ID"}'   --accountId $ID
```