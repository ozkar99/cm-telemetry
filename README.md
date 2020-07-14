# cm-telemetry

Implements the codemasters UDP telemetry protocol
and provides an abstraction for interacting with it.

## Supported Games

- Dirt Rally 2.0

### Using Externally Defined Games

You can support new games (external to the crate) by implementing the `Event` trait on your type and then initialize a `Server` as usual:

`cm_telemetry::Server::<T>::new()` to bind on the default (`127.0.0.1:20777`) address 
or `cm_telemetry::Server::<T>::with_addr()` if you want to bind to a different address/port.

### Furter Reading

https://forums.codemasters.com/topic/50942-f1-2020-udp-specification/
