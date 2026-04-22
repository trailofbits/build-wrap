use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    process::exit,
    time::Duration,
};

fn main() {
    let port = env::var("BUILD_WRAP_TCP_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or_else(|| {
            eprintln!("BUILD_WRAP_TCP_PORT is undefined or invalid");
            exit(1);
        });

    let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

    if let Err(error) = TcpStream::connect_timeout(&address.into(), Duration::from_secs(1)) {
        eprintln!("failed to connect to {address}: {error}");
        exit(1);
    }
}
