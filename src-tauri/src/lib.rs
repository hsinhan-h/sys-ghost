use serde::Serialize;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use sysinfo::{Networks, System, MINIMUM_CPU_UPDATE_INTERVAL};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemStats {
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
    network_download_bps: f64,
}

struct SysInfoState {
    system: System,
    networks: Networks,
    last_network_refresh: Option<Instant>,
    cpu_seeded: bool,
}

impl SysInfoState {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_memory();

        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            last_network_refresh: Some(Instant::now()),
            cpu_seeded: false,
        }
    }

    fn refresh_cpu_usage(&mut self) {
        if !self.cpu_seeded {
            self.system.refresh_cpu_all();
            std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
            self.cpu_seeded = true;
        }

        self.system.refresh_cpu_all();
    }

    fn refresh_network_download_bps(&mut self) -> f64 {
        let now = Instant::now();
        self.networks.refresh(true);

        let elapsed = self
            .last_network_refresh
            .map(|previous| now.saturating_duration_since(previous))
            .unwrap_or(Duration::ZERO);
        self.last_network_refresh = Some(now);

        if elapsed.is_zero() {
            return 0.0;
        }

        let downloaded_bytes = self
            .networks
            .iter()
            .map(|(_, network)| network.received())
            .sum::<u64>();

        downloaded_bytes as f64 / elapsed.as_secs_f64()
    }

    fn collect_stats(&mut self) -> SystemStats {
        self.system.refresh_memory();
        self.refresh_cpu_usage();
        let network_download_bps = self.refresh_network_download_bps();

        SystemStats {
            cpu_usage: self.system.global_cpu_usage(),
            memory_used: self.system.used_memory(),
            memory_total: self.system.total_memory(),
            network_download_bps,
        }
    }
}

struct AppState {
    sysinfo: Mutex<SysInfoState>,
}

#[tauri::command]
fn get_system_stats(state: tauri::State<'_, AppState>) -> Result<SystemStats, String> {
    let mut sysinfo = state
        .sysinfo
        .lock()
        .map_err(|_| "failed to acquire system stats lock".to_string())?;

    Ok(sysinfo.collect_stats())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            sysinfo: Mutex::new(SysInfoState::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_system_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
