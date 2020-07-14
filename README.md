# cm-telemetry

Implements the codemasters (and friends) UDP telemetry protocol
and provides an abstraction for interacting with it.

## Supported Games

- Dirt Rally 2.0

### Using Externally Defined Games

You can support new games (external to the crate) by implementing the `Event` trait on a type
and then initialize a `Server` with it using `cm_telemetry::Server::<T>::new(addr)`

PR's welcome :)

### Furter Reading

https://forums.codemasters.com/topic/50942-f1-2020-udp-specification/
