# Rust communication library for Lego Powered Up

## Example

```rust
// todo
```

## Architecture

Main components (tokio tasks):

* PoweredUp
  * Listener for Bluetooth device discovery notifications from btleplug
* HubManager
  * Owns the Peripherals corresponding to each hub
  * Listens for subscription messages and updates the stored hub & device states

Communication:
* Internal RPC structure
  * HubManager listens on a control channel
  * Requests down the control channel may include the sending half of a response channel
