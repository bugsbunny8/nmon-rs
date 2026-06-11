//! nmon-rs: A system performance monitor written in Rust.
//!
//! This crate provides a terminal user interface (TUI) and a CSV logging utility
//! to monitor system metrics including CPU, memory, disks, network, processes, and filesystems.

use chrono::Local;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use metrics::snapshot::{MetricSnapshot, DiskIoInfo, ProcessInfo, SystemResources};
use metrics::{
    cpu::CpuCollector, disk::DiskCollector, memory::MemoryCollector, network::NetworkCollector,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::{Duration, Instant};
use ui::dashboard::UiState;

mod alerting;
mod metrics;
mod ui;

/// Entry point of the program.
/// Parses command line arguments and routes execution to either interactive TUI mode or CSV logging mode.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. CLI Argument Parsing
    let args: Vec<String> = std::env::args().collect();
    let mut log_mode = false;
    let mut log_filename: Option<String> = None;
    let mut interval = 2;
    let mut count = 288; // Default 288 snapshots (e.g. 1 day with 5-min intervals)

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "-?" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("nmon-rs version 16s");
                return Ok(());
            }
            "-f" => {
                log_mode = true;
                i += 1;
            }
            "-F" => {
                if i + 1 < args.len() {
                    log_mode = true;
                    log_filename = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -F requires a filename argument.");
                    std::process::exit(1);
                }
            }
            "-s" => {
                if i + 1 < args.len() {
                    if let Ok(sec) = args[i + 1].parse::<u64>() {
                        interval = sec.max(1);
                    } else {
                        eprintln!("Error: -s requires an integer value.");
                        std::process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: -s requires a value.");
                    std::process::exit(1);
                }
            }
            "-c" => {
                if i + 1 < args.len() {
                    if let Ok(cnt) = args[i + 1].parse::<u64>() {
                        count = cnt.max(1);
                    } else {
                        eprintln!("Error: -c requires an integer value.");
                        std::process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: -c requires a value.");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown option: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    // 2. Initialize Collectors
    let mut cpu = CpuCollector::new();
    let mut mem = MemoryCollector::new();
    let mut disk = DiskCollector::new();
    let mut net = NetworkCollector::new();
    let mut sys_info = sysinfo::System::new_all();
    sys_info.refresh_all();

    // 3. Execution Routing
    if log_mode {
        // Run in CSV Logging Mode (non-interactive)
        let hostname = sysinfo::System::host_name().unwrap_or_else(|| "localhost".to_string());
        let filename = log_filename.unwrap_or_else(|| {
            let now = Local::now();
            format!("{}_{}.nmon", hostname, now.format("%y%m%d_%H%M"))
        });

        let mut file = std::fs::File::create(&filename)?;
        
        // Write CSV Metadata and Headers
        let snapshot = collect_snapshot(&mut cpu, &mut mem, &mut disk, &mut net, &mut sys_info);
        metrics::csv_logger::write_static_headers(&mut file, &snapshot, &args, interval, count)?;

        println!("Logging started. Saving stats to: {}", filename);

        for tick in 1..=count {
            std::thread::sleep(Duration::from_secs(interval));
            let snap = collect_snapshot(&mut cpu, &mut mem, &mut disk, &mut net, &mut sys_info);
            metrics::csv_logger::write_snapshot_row(&mut file, &snap, tick)?;
            
            // Asynchronously log alerts to alerts.log
            let snap_clone = snap.clone();
            std::thread::spawn(move || {
                alerting::handler::evaluate_alerts(&snap_clone);
            });
            
            println!("Recorded snapshot T{:04}/T{:04}", tick, count);
        }

        println!("Logging complete. Exiting.");
        Ok(())
    } else {
        // Run in Interactive TUI Mode
        let mut state = UiState {
            show_help: false,
            show_cpu: false,
            show_memory: false,
            show_disk: 0,
            show_filesystem: false,
            show_network: false,
            show_kernel: false,
            show_resources: false,
            show_processes: false,
            show_longterm: false,
            show_vm: false,
            show_diskmap: false,
            process_sort_by_cpu: true,
            refresh_interval_secs: interval,
            peak_disk_kb: std::cell::Cell::new(1024.0),
        };

        // Collect initial TUI snapshot
        let mut snapshot = collect_snapshot(&mut cpu, &mut mem, &mut disk, &mut net, &mut sys_info);

        let mut cpu_history = vec![ui::longterm_widget::CpuSnap {
            user: -1.0,
            sys: 0.0,
            wait: 0.0,
            idle: 0.0,
            steal: 0.0,
        }; 72];
        let initial_snap = ui::longterm_widget::CpuSnap {
            user: snapshot.cpu_global * 0.85,
            sys: snapshot.cpu_global * 0.15,
            wait: 0.0,
            idle: (100.0 - snapshot.cpu_global).max(0.0),
            steal: 0.0,
        };
        cpu_history[0] = initial_snap;
        let mut cpu_history_cursor = 1;

        // Setup terminal raw mode
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut last_tick = Instant::now();

        loop {
            // Redraw screen
            terminal.draw(|f| ui::dashboard::render(f, &snapshot, &state, &cpu_history, cpu_history_cursor))?;

            // Compute timeout for poll
            let elapsed = last_tick.elapsed();
            let timeout = Duration::from_secs(state.refresh_interval_secs)
                .checked_sub(elapsed)
                .unwrap_or(Duration::ZERO);

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('c') => state.show_cpu = !state.show_cpu,
                            KeyCode::Char('l') => state.show_longterm = !state.show_longterm,
                            KeyCode::Char('m') => state.show_memory = !state.show_memory,
                            KeyCode::Char('V') => state.show_vm = !state.show_vm,
                            KeyCode::Char('o') => state.show_diskmap = !state.show_diskmap,
                            KeyCode::Char('d') => {
                                state.show_disk = match state.show_disk {
                                    0 => 1,
                                    1 => 0,
                                    2 => 1,
                                    _ => 0,
                                };
                            }
                            KeyCode::Char('D') => {
                                state.show_disk = match state.show_disk {
                                    0 => 2,
                                    1 => 2,
                                    2 => 0,
                                    _ => 0,
                                };
                            }
                            KeyCode::Char('j') => state.show_filesystem = !state.show_filesystem,
                            KeyCode::Char('n') => state.show_network = !state.show_network,
                            KeyCode::Char('k') => state.show_kernel = !state.show_kernel,
                            KeyCode::Char('r') => state.show_resources = !state.show_resources,
                            KeyCode::Char('t') => state.show_processes = !state.show_processes,
                            KeyCode::Char('4') => state.process_sort_by_cpu = false,
                            KeyCode::Char('5') => state.process_sort_by_cpu = true,
                            KeyCode::Char('h') => state.show_help = !state.show_help,
                            KeyCode::Char('+') => {
                                state.refresh_interval_secs = (state.refresh_interval_secs * 2).min(64);
                            }
                            KeyCode::Char('-') => {
                                state.refresh_interval_secs = (state.refresh_interval_secs / 2).max(1);
                            }
                            KeyCode::Char(' ') => {
                                snapshot = collect_snapshot(&mut cpu, &mut mem, &mut disk, &mut net, &mut sys_info);
                                let snap_entry = ui::longterm_widget::CpuSnap {
                                    user: snapshot.cpu_global * 0.85,
                                    sys: snapshot.cpu_global * 0.15,
                                    wait: 0.0,
                                    idle: (100.0 - snapshot.cpu_global).max(0.0),
                                    steal: 0.0,
                                };
                                cpu_history[cpu_history_cursor] = snap_entry;
                                cpu_history_cursor = (cpu_history_cursor + 1) % 72;
                                last_tick = Instant::now();
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Refresh metrics if interval has elapsed
            if last_tick.elapsed() >= Duration::from_secs(state.refresh_interval_secs) {
                snapshot = collect_snapshot(&mut cpu, &mut mem, &mut disk, &mut net, &mut sys_info);
                
                let snap_entry = ui::longterm_widget::CpuSnap {
                    user: snapshot.cpu_global * 0.85,
                    sys: snapshot.cpu_global * 0.15,
                    wait: 0.0,
                    idle: (100.0 - snapshot.cpu_global).max(0.0),
                    steal: 0.0,
                };
                cpu_history[cpu_history_cursor] = snap_entry;
                cpu_history_cursor = (cpu_history_cursor + 1) % 72;

                // Log alerts asynchronously
                let snap_clone = snapshot.clone();
                std::thread::spawn(move || {
                    alerting::handler::evaluate_alerts(&snap_clone);
                });

                last_tick = Instant::now();
            }
        }

        // Restore terminal state
        disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
}

/// Prints the command line usage help menu to standard output.
fn print_help() {
    println!("nmon-rs: Curses based Performance Monitor (written in Rust)");
    println!("Usage: monitor-rs [options]");
    println!();
    println!("Options:");
    println!("  -f          Start CSV logging mode to a default named file.");
    println!("  -F <file>   Start CSV logging mode to the specified file.");
    println!("  -s <secs>   Set snapshot refresh interval in seconds (default: 2).");
    println!("  -c <count>  Set the snapshot count (default: 288, logging mode mode only).");
    println!("  -h, -?      Display this help menu and exit.");
    println!("  -V          Print version and exit.");
}

/// Collects a complete system resource usage snapshot by querying CPU, memory, disk, network collectors, and system info.
fn collect_snapshot(
    cpu: &mut CpuCollector,
    mem: &mut MemoryCollector,
    disk: &mut DiskCollector,
    net: &mut NetworkCollector,
    sys_info: &mut sysinfo::System,
) -> MetricSnapshot {
    sys_info.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "localhost".to_string());
    let os_name = sysinfo::System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = sysinfo::System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let cpu_arch = sysinfo::System::cpu_arch();
    
    let uptime_secs = sysinfo::System::uptime();
    let load_avg = sysinfo::System::load_average();

    let (cpu_global, cpu_cores) = cpu.collect();
    let cpu_vendor = cpu.cpu_vendor();
    let cpu_model = cpu.cpu_brand();
    let physical_cores = cpu.physical_cores();
    let logical_cores = cpu.logical_cores();

    let memory = mem.collect();
    let (read_bps, write_bps) = disk.collect_io();
    let filesystems = disk.collect_filesystems();
    let networks = net.collect();

    let mut processes = Vec::new();
    for (pid, process) in sys_info.processes() {
        let cmd = process
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        let name_str = process.name().to_string_lossy().into_owned();
        let display_cmd = if cmd.is_empty() { name_str.clone() } else { cmd };
        processes.push(ProcessInfo {
            pid: pid.to_string().parse::<u32>().unwrap_or(0),
            name: name_str,
            cpu_usage: process.cpu_usage(),
            memory_bytes: process.memory(),
            disk_read_bps: process.disk_usage().total_read_bytes,
            disk_write_bps: process.disk_usage().total_written_bytes,
            command: display_cmd,
        });
    }

    MetricSnapshot {
        timestamp: Local::now(),
        cpu_global,
        cpu_cores,
        memory,
        disk_io: DiskIoInfo {
            read_bps,
            write_bps,
        },
        filesystems,
        networks,
        processes,
        system: SystemResources {
            hostname,
            os_name,
            os_version,
            kernel_version,
            cpu_arch,
            cpu_vendor,
            cpu_model,
            physical_cores,
            logical_cores,
            uptime_secs,
            load_avg_1m: load_avg.one,
            load_avg_5m: load_avg.five,
            load_avg_15m: load_avg.fifteen,
        },
    }
}
