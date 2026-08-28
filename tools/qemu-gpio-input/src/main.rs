use device_query::{DeviceQuery, DeviceState, Keycode};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread::sleep;
use std::time::Duration;

const HOST_PORT: &str = "127.0.0.1:5000";
const GPIO_BASE: u32 = 0xfe200000;

fn main() -> std::io::Result<()> {
    let device_state = DeviceState::new();
    let key_map = vec![
        (Keycode::W, 0),
        (Keycode::A, 1),
        (Keycode::S, 2),
        (Keycode::D, 3),
        (Keycode::I, 4),
        (Keycode::J, 5),
        (Keycode::K, 6),
        (Keycode::L, 7),
        (Keycode::F, 25),
        (Keycode::H, 9),
    ];
    let mut states: HashMap<u8, bool> = key_map.iter().map(|&(_, pin)| (pin, false)).collect();

    let addr: SocketAddr = HOST_PORT.parse().unwrap();
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
    stream.set_nonblocking(true)?;
    let mut buf = [0u8; 1024];

    loop {
        match stream.peek(&mut buf) {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
            _ => {}
        }

        let keys = device_state.get_keys();

        for &(keycode, pin) in &key_map {
            let is_pressed = keys.contains(&keycode);
            let current_state = states.get_mut(&pin).unwrap();

            if is_pressed != *current_state {
                let offset = if is_pressed { 0x1c } else { 0x28 };
                let address = GPIO_BASE + offset;
                let value = 1 << pin;
                let msg = format!("writel 0x{address:x} 0x{value:x}\n");

                stream.set_nonblocking(false)?;
                stream.write_all(msg.as_bytes())?;
                let _ = stream.read(&mut buf);
                stream.set_nonblocking(true)?;

                *current_state = is_pressed;
            }
        }

        sleep(Duration::from_millis(10));
    }

    Ok(())
}
