use super::BalancingAlgorithm;
use super::HostManager;

pub struct RoundRobin {
    current_host: usize,
    max_host: usize,
    host_manager: HostManager,
}

impl RoundRobin {
    pub fn new(host_manager: HostManager) -> Self {
        let max = host_manager.hosts.len();
        RoundRobin {
            current_host: 0,
            host_manager: host_manager,
            max_host: max,
        }
    }
}

impl BalancingAlgorithm for RoundRobin {
    fn get_next_host(&mut self) -> String {
        let val = self.host_manager.hosts[self.current_host].to_string();

        self.current_host = self.current_host + 1;
        if self.current_host > self.max_host {
            self.current_host = 0
        }

        val
    }
}
