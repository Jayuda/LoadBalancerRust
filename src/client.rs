use std::io::prelude::*;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

pub struct TcpClient {
    stream: TcpStream,
    buffer: [u8; 4096],

    target: Option<SocketAddr>,
    target_stream: Option<TcpStream>,
    is_connected: bool,
    pub address: SocketAddr,
}

impl TcpClient {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        let addr = stream.peer_addr().unwrap();
        println!("Connected from {}", addr.to_string());

        TcpClient {
            stream: stream,
            buffer: [0; 4096],
            target: None,
            target_stream: None,
            address: addr,
            is_connected: false,
        }
    }

    pub fn connect_to_target(&mut self, target: SocketAddr, timeout: Duration) {
        self.close_connection_to_target();

        let str = match TcpStream::connect_timeout(&target, timeout) {
            Ok(stream) => stream,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // timed out
                println!(
                    "[{} <-> {}] Timed out connection to server",
                    self.address, target
                );
                return;
            }
            Err(err) => {
                println!(
                    "[{} <-> {}] Error while trying to connect to server: {}",
                    self.address,
                    target,
                    err.to_string()
                );
                return;
            }
        };

        str.set_nonblocking(true).unwrap();

        self.target = Some(target);
        self.target_stream = Some(str);
        self.is_connected = true;

        println!("[{} <-> {}] Connection established", self.address, target);
    }

    pub fn process(&mut self) {
        // do not process if target host is not set/connected
        let target = match self.target {
            Some(t) => t,
            None => return,
        };

        let mut str = self.target_stream.as_ref().unwrap();

        // READ FROM CLIENT
        let read: i32 = match self.stream.read(&mut self.buffer) {
            Ok(r) => r as i32,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => -1,
            Err(err) => {
                // error with connection
                println!(
                    "[{} <-> {}] Connection to client failed!",
                    self.address, target
                );
                return;
            }
        };

        // WRITE TO SERVER
        if read > 0 {
            str.write(&self.buffer[..(read as usize)]).unwrap();
        } else if read == 0 {
            println!("[{} <-> {}] Zero buffer from client", self.address, target);
            return;
        }

        // READ FROM SERVER
        let reads: i32 = match str.read(&mut self.buffer) {
            Ok(r) => r as i32,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => -1,
            Err(err) => {
                // error with connection
                println!(
                    "[{} <-> {}] Connection to server failed!",
                    self.address, target
                );
                return;
            }
        };

        // WRITE TO CLIENT
        if reads > 0 {
            self.stream.write(&self.buffer[..(reads as usize)]).unwrap();
        } else if reads == 0 {
            println!("[{} <-> {}] Zero buffer from server", self.address, target);
            return;
        }
    }

    fn close_connection_to_target(&mut self) {
        if self.is_connected {
            let str = self.target_stream.as_ref().unwrap();
            str.shutdown(Shutdown::Both)
                .expect("Failed to shutdown server TCP stream");

            self.is_connected = false;

            println!(
                "[{} <-> {}] Connection ended",
                self.address,
                self.target.unwrap()
            );
        }
    }
}

impl Drop for TcpClient {
    fn drop(&mut self) {
        self.stream
            .shutdown(Shutdown::Both)
            .expect("Failed to shutdown client TCP stream");

        if self.is_connected {
            self.close_connection_to_target();
        } else {
            println!("[{}] Connection ended", self.address);
        }
    }
}
