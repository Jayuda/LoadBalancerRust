use std::io::Result;
use std::process::exit;

mod balancer;
use balancer::Poller;
use balancer::RoundRobin;
use balancer::{HostManager, LoadBalancer};

fn main() -> Result<()> {
    // PARSE HOSTS
    let host_manager = HostManager::new("hosts");
    if host_manager.hosts.len() == 0 {
        return Ok(());
    }

    // INITIALIZE
    let debug_mode = true;
    let round_robin = RoundRobin::new(host_manager);
    let balancer = LoadBalancer::new(round_robin, 4, debug_mode);
    let mut poller = Poller::new(balancer);

    // PARSE PORT
    let port = match get_port() {
        Some(p) => p,
        None => {
            println!("Invalid listening port provided, use default 4554");
            4554
        }
    };

    // START
    poller.start_listening(port).unwrap_or_else(|e| {
        println!("{}", e.to_string());
        exit(2);
    });

    Ok(())
}

fn get_port() -> Option<i32> {
    let listening_port = std::env::args().nth(1)?;
    let port: i32 = match listening_port.parse() {
        Ok(p) => p,
        Err(_) => return None,
    };

    if port <= 0 || port > 65535 {
        return None;
    }

    Some(port)
}
