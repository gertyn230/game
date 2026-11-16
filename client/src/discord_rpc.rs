use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DiscordRpc {
    client: Option<DiscordIpcClient>,
    start_timestamp: i64,
}

impl DiscordRpc {
    pub fn new() -> Self {
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut client = match DiscordIpcClient::new("1234567890123456789") {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Failed to create Discord RPC client: {}", e);
                None
            }
        };

        if let Some(ref mut c) = client {
            if let Err(e) = c.connect() {
                eprintln!("Failed to connect to Discord: {}", e);
                client = None;
            }
        }

        Self {
            client,
            start_timestamp,
        }
    }

    pub fn update(&mut self, state: &str, details: &str, players_online: Option<usize>) {
        if let Some(ref mut client) = self.client {
            let mut activity_builder = activity::Activity::new()
                .state(state)
                .details(details)
                .timestamps(
                    activity::Timestamps::new().start(self.start_timestamp)
                )
                .assets(
                    activity::Assets::new()
                        .large_image("pixel_village_logo")
                        .large_text("Pixel Village - Build your village!")
                );

            if let Some(count) = players_online {
                activity_builder = activity_builder.party(
                    activity::Party::new()
                        .size([1, count as i32])
                );
            }

            if let Err(e) = client.set_activity(activity_builder) {
                eprintln!("Failed to update Discord activity: {}", e);
            }
        }
    }

    pub fn clear(&mut self) {
        if let Some(ref mut client) = self.client {
            let _ = client.clear_activity();
        }
    }
}

impl Drop for DiscordRpc {
    fn drop(&mut self) {
        self.clear();
        if let Some(ref mut client) = self.client {
            let _ = client.close();
        }
    }
}
