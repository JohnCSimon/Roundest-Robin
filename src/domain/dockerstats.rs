use bollard::container::{ListContainersOptions, StatsOptions};
use bollard::secret::ContainerSummary;
use bollard::Docker;
use bytesize::ByteSize;
use futures_util::StreamExt;
use std::collections::HashMap;

pub struct DockerStats {
    pub cpu_percentage: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percentage: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

pub async fn get_docker_stats() -> Result<HashMap<String, DockerStats>, Box<dyn std::error::Error>>
{
    let docker = Docker::connect_with_unix(
        "/Users/johnsimon/.docker/run/docker.sock",
        120, // timeout seconds
        bollard::API_DEFAULT_VERSION,
    )?;

    // list all running containers
    let running = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: false, // false = running only
            ..Default::default()
        }))
        .await?;

    let mut container_stats: HashMap<String, DockerStats> = HashMap::new();
    for container in running.iter() {
        let result = print_stats_per_container(container, &docker).await?;
        let port = container
            .image
            .as_ref()
            .map(|img| img.split('-').last().unwrap_or(""))
            .unwrap_or("");
        let url = format!("http://localhost:{}/", port);
        container_stats.insert(url, result);
    }

    Ok(container_stats)
}

async fn print_stats_per_container(
    first: &ContainerSummary,
    docker: &Docker,
) -> Result<DockerStats, Box<dyn std::error::Error>> {
    let binding = first.id.clone().unwrap();
    let container_id = first.id.as_ref().unwrap_or(&binding);
    // Docker gives long IDs; name is often easier to read:
    let display_name = first
        .names
        .as_ref()
        .and_then(|names| names.get(0))
        .map(|s| s.trim_start_matches('/'))
        .unwrap_or(container_id);

    println!(
        "Inspecting stats for container: {} ({})",
        display_name, container_id
    );

    //
    // 3. Request stats.
    //
    // docker.stats() returns an async stream of Stat objects.
    // stream=true means continuous stream. We'll just grab the first item and stop.
    //
    let mut stats_stream = docker.stats(
        container_id,
        Some(StatsOptions {
            stream: false,
            one_shot: true, // some dockerds support this flag, bollard may expose it as `one_shot`
        }),
    );

    // Pull exactly one stats frame
    let stats_frame = stats_stream
        .next()
        .await
        .ok_or("docker returned no stats")??;

    //
    // 4. Extract interesting fields.
    //
    // The docker API gives cpu_stats and precpu_stats so you can compute deltas.
    // We'll calculate CPU % the same way docker CLI does:
    //
    // CPU% = (cpu_delta / system_delta) * #cpus * 100.0
    //
    let cpu_pct = calc_cpu_percent(&stats_frame);

    // Memory usage vs limit
    let mem_usage = stats_frame.memory_stats.usage.unwrap_or_default() as u64;
    let mem_limit = stats_frame.memory_stats.limit.unwrap_or_default() as u64;

    // Network rx/tx (can be multiple interfaces; sum them)
    let (rx_bytes, tx_bytes) = calc_network_io(&stats_frame);

    //
    // 5. Print it.
    //
    println!("\n=== Live stats ===");
    println!("Name:            {}", display_name);
    println!("CPU usage:       {:.2}%", cpu_pct);
    println!(
        "Memory usage:    {} / {} ({:.2}%)",
        ByteSize(mem_usage),
        ByteSize(mem_limit),
        pct(mem_usage, mem_limit),
    );
    println!(
        "Network I/O:     ↓ {} / ↑ {}",
        ByteSize(rx_bytes),
        ByteSize(tx_bytes),
    );

    let stats = DockerStats {
        cpu_percentage: cpu_pct,
        memory_usage: mem_usage,
        memory_limit: mem_limit,
        memory_percentage: pct(mem_usage, mem_limit),
        network_rx_bytes: rx_bytes,
        network_tx_bytes: tx_bytes,
    };
    // blkio, pids, etc. are also available on stats_frame if you need them.
    Ok(stats)
}

/// Calculate the CPU percentage in the Docker CLI style.
///
/// Formula:
///   cpu_delta = cpu_stats.cpu_usage.total_usage - precpu_stats.cpu_usage.total_usage
///   system_delta = cpu_stats.system_cpu_usage - precpu_stats.system_cpu_usage
///   cpu_percent = (cpu_delta / system_delta) * cpu_stats.online_cpus * 100.0
fn calc_cpu_percent(stats: &bollard::container::Stats) -> f64 {
    let cpu_stats = &stats.cpu_stats;
    let precpu_stats = &stats.precpu_stats;

    let cpu_total = cpu_stats.cpu_usage.total_usage as f64;
    let pre_cpu_total = precpu_stats
        .cpu_usage
        .percpu_usage
        .as_ref()
        .map(|v| v.iter().sum::<u64>() as f64)
        .unwrap_or(0.0);

    let cpu_delta = cpu_total - pre_cpu_total;

    let system_cpu = cpu_stats.system_cpu_usage.unwrap_or(0) as f64;
    let pre_system_cpu = precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

    let system_delta = system_cpu - pre_system_cpu;

    let online_cpus = cpu_stats.online_cpus.unwrap_or_else(|| {
        cpu_stats
            .cpu_usage
            .percpu_usage
            .as_ref()
            .map(|v| v.len() as u64)
            .unwrap_or(1)
    }) as f64;

    if system_delta > 0.0 && cpu_delta > 0.0 {
        (cpu_delta / system_delta) * online_cpus * 100.0
    } else {
        0.0
    }
}

/// Sum RX/TX bytes across all reported network interfaces.
fn calc_network_io(stats: &bollard::container::Stats) -> (u64, u64) {
    if let Some(networks) = &stats.networks {
        let mut rx = 0u64;
        let mut tx = 0u64;
        for (_ifname, data) in networks {
            rx += data.rx_bytes;
            tx += data.tx_bytes;
        }
        (rx, tx)
    } else {
        (0, 0)
    }
}

fn pct(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64) * 100.0
    }
}
