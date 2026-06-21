use std::process::Command;
use sysinfo::{Disks, Networks, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Default)]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub cpu_cores: Vec<f32>,
    pub cpu_total: f64,
    pub cpu_brand: String,
    pub gpu_name: String,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub net_rx_kbps: f64,
    pub net_tx_kbps: f64,
    pub disks: Vec<DiskInfo>,
    pub disk_read_kbps: f64,
    pub disk_write_kbps: f64,
}

fn query_gpu_name() -> String {
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_VideoController).Name",
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .map(|l| l.trim())
                .find(|l| !l.is_empty())
                .map(|l| l.to_string())
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "N/A".to_string())
}

pub async fn collect_loop(tx: Sender<Metrics>) {
    let mut sys = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();
    let mut disks = Disks::new_with_refreshed_list();
    let mut ticker = interval(Duration::from_secs(1));
    let gpu_name = query_gpu_name();

    let mut last_rx: u64 = 0;
    let mut last_tx: u64 = 0;
    let mut primed = false;

    let proc_refresh = ProcessRefreshKind::new().with_disk_usage();

    loop {
        ticker.tick().await;
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, proc_refresh);
        networks.refresh();
        disks.refresh();

        let cores: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let cpu_total =
            cores.iter().map(|&v| v as f64).sum::<f64>() / cores.len().max(1) as f64;
        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_default();

        let (total_rx, total_tx) = networks.iter().fold((0u64, 0u64), |(r, t), (_, d)| {
            (r + d.total_received(), t + d.total_transmitted())
        });

        let disk_info: Vec<DiskInfo> = disks
            .list()
            .iter()
            .map(|d| {
                let total = d.total_space();
                let avail = d.available_space();
                DiskInfo {
                    name: d.name().to_string_lossy().into_owned(),
                    mount: d.mount_point().to_string_lossy().into_owned(),
                    used_bytes: total.saturating_sub(avail),
                    total_bytes: total,
                }
            })
            .collect();

        let (delta_rx, delta_tx) = if primed {
            (
                total_rx.saturating_sub(last_rx),
                total_tx.saturating_sub(last_tx),
            )
        } else {
            primed = true;
            (0, 0)
        };
        last_rx = total_rx;
        last_tx = total_tx;

        let (read_bytes, written_bytes) =
            sys.processes()
                .iter()
                .fold((0u64, 0u64), |(r, w), (_, p)| {
                    let du = p.disk_usage();
                    (r + du.read_bytes, w + du.written_bytes)
                });

        let m = Metrics {
            cpu_total,
            cpu_cores: cores,
            cpu_brand,
            gpu_name: gpu_name.clone(),
            ram_used_mb: sys.used_memory() / 1024 / 1024,
            ram_total_mb: sys.total_memory() / 1024 / 1024,
            net_rx_kbps: delta_rx as f64 / 1024.0,
            net_tx_kbps: delta_tx as f64 / 1024.0,
            disks: disk_info,
            disk_read_kbps: read_bytes as f64 / 1024.0,
            disk_write_kbps: written_bytes as f64 / 1024.0,
        };

        if tx.send(m).await.is_err() {
            break;
        }
    }
}
