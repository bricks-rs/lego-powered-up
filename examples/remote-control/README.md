# remote-control

This example code connects to the first hub it finds; assumes it is a
remote handset (no checking in this example).
It then prints updates when remote buttons are pressed and released.

Note: The handset provides only a single event for any button released
within each button cluster (A- and B-sides are differentiated by port).
For example:
    Press and hold A+    -> A+ pressed event
    Press and hold A-    -> A- pressed event
    Release either       -> Button released event
    Release other        -> No event  

Ctrl-C to disconnect and exit.

## Usage

```bash
cargo run --package remote-handset
```

## License

The code in this example is public domain and may be used/modified without permission or attribution.
