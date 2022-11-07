# Drogue IoT ESP8266

[![crates.io](https://img.shields.io/crates/v/drogue-esp8266.svg)](https://crates.io/crates/drogue-esp8266)
[![docs.rs](https://docs.rs/drogue-esp8266/badge.svg)](https://docs.rs/drogue-esp8266)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

A network driver for an ESP8266 attached via a USART. See [esp8266-at-driver](https://github.com/drogue-iot/esp8266-at-driver) for a driver that
works with async Rust.

Currently requires the ESP to be flashed with a 1.7.0.x version of the AT firmware provided by Espressif.

To use, you must configure your USART as 115,200 bps and 8-N-1, along with selecting the `enable` and `reset` connections to the board.

By using the `initialize(...)` function, you will get a 2-tuple back, container the `Adapter` and an `Ingress` object:

```rust
static mut RESPONSE_QUEUE: Queue<Response, U2> = Queue(i::Queue::new());
static mut NOTIFICATION_QUEUE: Queue<Response, U16> = Queue(i::Queue::new());

let (adapter, ingress) = esp8266::initialize(
    tx, rx,
    &mut en, &mut reset,
    unsafe { &mut RESPONSE_QUEUE },
    unsafe { &mut NOTIFICATION_QUEUE },
).unwrap();
```

In an RTIC app, this would occur during the init phase of the app, and both pieces would be placed into the shared resources.

The `Ingress` should be wired up to the USART interrupt in order to receive octets from the serial port:

```rust
#[task(binds = USART6, priority = 10, resources = [ingress])]
fn usart(ctx: usart::Context) {
    if let Err(b) = ctx.resources.ingress.isr() {
        info!("failed to ingress {}", b as char);
    }
}
```

Additionally, the `Ingress` should be attached to a timer loop in order to process all received octets in a timely fashion. 
The cycle speed is left as an exercise for the reader:

```rust
#[task(schedule = [digest], priority = 2, resources = [ingress])]
fn digest(mut ctx: digest::Context) {
    ctx.resources.ingress.lock(|ingress| ingress.digest());
    ctx.schedule.digest(ctx.scheduled + (DIGEST_DELAY * 100_000).cycles())
        .unwrap();
}
```

Once all iterrupts/tasks are enabled, the adapter may then be used in order to join a Wifi access point:

```rust
let result = adapter.join("myaccesspoint", "thepassword");
```

After successfully joining, the adapter may be convereted into a `TCPNetworkStack`:

```rust
 let network = adapter.into_network_stack();

 let socket = network.open(Mode::Blocking).unwrap();

 let socket_addr = SocketAddr::new(
     IpAddr::from_str("192.168.1.245").unwrap(),
     80,
 );

 let mut socket = network.connect(socket, socket_addr).unwrap();
 let result = network.write(&mut socket, b"GET / HTTP/1.1\r\nhost:192.168.1.245\r\n\r\n").unwrap();

 loop {
     let mut buffer = [0; 128];
     let result = network.read(&mut socket, &mut buffer);
     match result {
         Ok(len) => {
             if len > 0 {
                 let s = core::str::from_utf8(&buffer[0..len]);
                 match s {
                     Ok(s) => {
                         info!("recv: {} ", s);
                     }
                     Err(_) => {
                         info!("recv: {} bytes (not utf8)", len);
                     }
                 }
             }
         }
         Err(e) => {
             info!("ERR: {:?}", e);
             break;
         }
     }
 }
```
