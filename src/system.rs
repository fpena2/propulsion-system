use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, info};

#[derive(Debug, Clone, Copy)]
pub enum CountdownState {
    Cancel,
    Start(u64),
}

pub struct MissionComputer {
    rx: watch::Receiver<CountdownState>,
    counter_task: Option<tokio::task::JoinHandle<()>>,
}

impl MissionComputer {
    pub fn new(rx: watch::Receiver<CountdownState>) -> Self {
        MissionComputer {
            rx,
            counter_task: None,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.rx.changed().await?;
            let command = *self.rx.borrow();
            self.handle_command(command).await;
        }
    }

    async fn handle_command(&mut self, command: CountdownState) {
        if let Some(task) = self.counter_task.take() {
            task.abort();
        }
        match command {
            CountdownState::Cancel => {
                debug!("Countdown cancelled");
            }
            CountdownState::Start(seconds) => {
                debug!("Starting new countdown: {} seconds", seconds);
                self.counter_task = Some(tokio::spawn(Self::fire_propulsion(seconds)));
            }
        }
    }

    async fn fire_propulsion(seconds: u64) {
        tokio::time::sleep(Duration::from_secs(seconds)).await;
        info!("firing now!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_handle_command_start_creates_task() {
        let (_tx, rx) = watch::channel(CountdownState::Cancel);
        let mut computer = MissionComputer::new(rx);

        computer.handle_command(CountdownState::Start(1)).await;
        assert!(computer.counter_task.is_some());
    }

    #[tokio::test]
    async fn test_handle_command_cancel_clears_task() {
        let (_tx, rx) = watch::channel(CountdownState::Cancel);
        let mut computer = MissionComputer::new(rx);

        computer.handle_command(CountdownState::Start(1)).await;
        assert!(computer.counter_task.is_some());

        computer.handle_command(CountdownState::Cancel).await;
        assert!(computer.counter_task.is_none());
    }
}
