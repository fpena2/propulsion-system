use std::time::Duration;
use tokio::sync::watch;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountdownCommand {
    Cancel,
    Start(u64),
    Reset,
    Invalid,
}

// Start Command
//  - Only starts if the propulsion system has not been yet fired.
// Reset Command
//  - Allow starting the propulsion system even if it has been previosuly been fired.
// Cancel Command
//  - Cancel the current countdown
pub struct MissionComputer {
    rx: watch::Receiver<CountdownCommand>,
    counter_task: Option<(u64, tokio::task::JoinHandle<()>)>, // countdown and handle
}

impl MissionComputer {
    pub fn new(rx: watch::Receiver<CountdownCommand>) -> Self {
        MissionComputer {
            rx,
            counter_task: None,
        }
    }

    pub async fn run(&mut self) {
        while self.rx.changed().await.is_ok() {
            let command = *self.rx.borrow();
            self.handle_command(command).await;
        }
    }

    async fn handle_command(&mut self, command: CountdownCommand) {
        match command {
            CountdownCommand::Cancel => {
                if let Some((_seconds, task)) = self.counter_task.as_ref() {
                    if !task.is_finished() {
                        task.abort();
                        info!("Countdown cancelled");
                    } else {
                        info!("Last countdown already ended");
                    }
                } else {
                    info!("No countdown to cancel");
                }
            }
            CountdownCommand::Start(seconds) => {
                if self.counter_task.is_none() {
                    info!("Starting new countdown: {} seconds", seconds);
                    self.counter_task =
                        Some((seconds, tokio::spawn(Self::fire_propulsion(seconds))));
                } else {
                    info!("Propulsion has already been fired");
                }
            }
            CountdownCommand::Reset => {
                if let Some((_seconds, task)) = self.counter_task.take() {
                    task.abort();
                    info!("Countdown has been reset");
                } else {
                    info!("No countdown to reset");
                }
            }
            _ => {
                info!("Received invalid command");
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
        let (_tx, rx) = watch::channel(CountdownCommand::Cancel);
        let mut computer = MissionComputer::new(rx);

        computer.handle_command(CountdownCommand::Start(1)).await;
        assert!(computer.counter_task.is_some());
    }

    #[tokio::test]
    async fn test_handle_command_reset_clears_task() {
        let (_tx, rx) = watch::channel(CountdownCommand::Cancel);
        let mut computer = MissionComputer::new(rx);

        computer.handle_command(CountdownCommand::Start(1)).await;
        assert!(computer.counter_task.is_some());

        computer.handle_command(CountdownCommand::Cancel).await;
        assert!(computer.counter_task.is_some());

        computer.handle_command(CountdownCommand::Reset).await;
        assert!(computer.counter_task.is_none());
    }
}
