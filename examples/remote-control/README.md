# remote-control

This example code connects to the first hub it finds; assumes it is a
remote handset (no checking in this example).
It then prints updates when remote buttons are pressed and released.
Ctrl-C to disconnect and exit.

## Hardware limitations

There are some limitations to take into account when designing your
remote control UI:

The handset provides only a single event for any button released
within each button cluster (A- and B-sides are differentiated by port).
Example:
    Press and hold A+    -> A+ pressed event
    Press and hold A-    -> A- pressed event
    Release either       -> Button released event
    Release other        -> No event  

While red button is pressed, no events are sent for + and - buttons.
Example:
    Press and hold Ared  -> Ared pressed event
    Press A+ or A-       -> No event
    Release Ared         -> Button released event
Example:
    Press and hold A+    -> A+ pressed event
    Press and hold Ared  -> Ared pressed event
    Release A+           -> No event
    Release Ared         -> Button released event  

As with other hubs the green button can be used for user interactions
with short presses; long press will disconnect the remote/hub. These
events are not sent on a port but instead as HwNetworkCommand events.

## Usage

```bash
cargo run --package remote-handset
```

## License

The code in this example is public domain and may be used/modified without permission or attribution.
