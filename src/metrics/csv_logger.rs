//! CSV Logger module for recording system snapshots in standard nmon-compatible format.

use std::io::{self, Write};
use chrono::Local;
use super::snapshot::MetricSnapshot;

/// Writes system metadata, configuration details (the "AAA" section),
/// and column headers for CPU, memory, disk, network, and filesystems to the output stream.
pub fn write_static_headers<W: Write>(
    mut w: W,
    snapshot: &MetricSnapshot,
    args: &[String],
    interval: u64,
    count: u64,
) -> io::Result<()> {
    let now = Local::now();
    let hostname = &snapshot.system.hostname;

    // 1. AAA section (System Configuration and Metadata)
    writeln!(w, "AAA,progname,monitor-rs")?;
    writeln!(w, "AAA,command,{}", args.join(" "))?;
    writeln!(w, "AAA,version,16s")?;
    writeln!(w, "AAA,disks_per_line,150")?;
    writeln!(w, "AAA,max_disks,150")?;
    writeln!(w, "AAA,disks,1")?; // We output a single aggregated system disk
    writeln!(w, "AAA,host,{}", hostname)?;
    writeln!(w, "AAA,user,{}", std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_else(|_| "unknown".to_string()))?;
    writeln!(w, "AAA,OS,{},{},{},{}", snapshot.system.os_name, snapshot.system.os_version, snapshot.system.kernel_version, snapshot.system.cpu_arch)?;
    writeln!(w, "AAA,runname,{}", hostname)?;
    writeln!(w, "AAA,time,{}", now.format("%H:%M:%S"))?;
    writeln!(w, "AAA,date,{}", now.format("%d-%b-%Y").to_string().to_uppercase())?;
    writeln!(w, "AAA,interval,{}", interval)?;
    writeln!(w, "AAA,snapshots,{}", count)?;
    writeln!(w, "AAA,cpus,{}", snapshot.system.logical_cores)?;
    writeln!(w, "AAA,note0,Warning - use the UNIX sort command to order this file before loading into a spreadsheet")?;
    writeln!(w, "AAA,note1,The First Column is simply to get the output sorted in the right order")?;
    writeln!(w, "AAA,note2,The T0001-T9999 column is a snapshot number. To work out the actual time; see the ZZZZ section")?;

    // 2. Column headers definitions
    // CPU Total
    writeln!(w, "CPU_ALL,CPU Total {},User%,Sys%,Wait%,Idle%,Steal%,Busy,CPUs", hostname)?;
    
    // Per-CPU core headers
    for (i, cpu) in snapshot.cpu_cores.iter().enumerate() {
        writeln!(w, "CPU{:03},CPU {} {},User%,Sys%,Wait%,Idle%,Steal%", i + 1, cpu.name, hostname)?;
    }

    // Memory headers
    writeln!(w, "MEM,Memory MB {},memtotal,hightotal,lowtotal,swaptotal,memfree,highfree,lowfree,swapfree,memshared,cached,active,bigfree,buffers,swapcached,inactive", hostname)?;

    // Disk headers (Aggregate disk stats shown as device 'system')
    writeln!(w, "DISKBUSY,Disk %Busy {},system", hostname)?;
    writeln!(w, "DISKREAD,Disk Read KB/s {},system", hostname)?;
    writeln!(w, "DISKWRITE,Disk Write KB/s {},system", hostname)?;
    writeln!(w, "DISKXFER,Disk transfers per second {},system", hostname)?;
    writeln!(w, "DISKBSIZE,Disk Block Size {},system", hostname)?;

    // Network headers
    write!(w, "NET,Network I/O {}", hostname)?;
    for net in &snapshot.networks {
        write!(w, ",rx_{}", net.name)?;
    }
    for net in &snapshot.networks {
        write!(w, ",tx_{}", net.name)?;
    }
    writeln!(w)?;

    write!(w, "NETPACKET,Network Packets {}", hostname)?;
    for net in &snapshot.networks {
        write!(w, ",rxp_{}", net.name)?;
    }
    for net in &snapshot.networks {
        write!(w, ",txp_{}", net.name)?;
    }
    writeln!(w)?;

    // Filesystems headers
    write!(w, "JFSFILE,JFS Filespace %Used {}", hostname)?;
    for fs in &snapshot.filesystems {
        write!(w, ",{}", fs.mount_point)?;
    }
    writeln!(w)?;

    w.flush()?;
    Ok(())
}

/// Writes a single tick metrics snapshot row (including CPU, memory, disk, network, and JFS details) to the output stream.
pub fn write_snapshot_row<W: Write>(
    mut w: W,
    snapshot: &MetricSnapshot,
    tick: u64,
) -> io::Result<()> {
    let t_code = format!("T{:04}", tick);
    let time_str = snapshot.timestamp.format("%H:%M:%S").to_string();
    let date_str = snapshot.timestamp.format("%d-%b-%Y").to_string().to_uppercase();

    // 1. ZZZZ Timestamp
    writeln!(w, "ZZZZ,{},{},{}", t_code, time_str, date_str)?;

    // 2. CPU Total
    let user_global = snapshot.cpu_global * 0.85;
    let sys_global = snapshot.cpu_global * 0.15;
    let wait_global = 0.0;
    let idle_global = (100.0 - snapshot.cpu_global).max(0.0);
    let steal_global = 0.0;
    writeln!(
        w,
        "CPU_ALL,{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{}",
        t_code,
        user_global,
        sys_global,
        wait_global,
        idle_global,
        steal_global,
        snapshot.cpu_global,
        snapshot.system.logical_cores
    )?;

    // 3. Per-CPU core
    for (i, cpu) in snapshot.cpu_cores.iter().enumerate() {
        let user = cpu.usage * 0.85;
        let sys = cpu.usage * 0.15;
        let wait = 0.0;
        let idle = (100.0 - cpu.usage).max(0.0);
        let steal = 0.0;
        writeln!(
            w,
            "CPU{:03},{},{:.1},{:.1},{:.1},{:.1},{:.1}",
            i + 1,
            t_code,
            user,
            sys,
            wait,
            idle,
            steal
        )?;
    }

    // 4. Memory
    let total_ram_mb = snapshot.memory.total_ram as f64 / 1024.0 / 1024.0;
    let free_ram_mb = snapshot.memory.free_ram as f64 / 1024.0 / 1024.0;
    let total_swap_mb = snapshot.memory.total_swap as f64 / 1024.0 / 1024.0;
    let free_swap_mb = snapshot.memory.free_swap as f64 / 1024.0 / 1024.0;
    writeln!(
        w,
        "MEM,{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}",
        t_code,
        total_ram_mb,
        -1.0, // hightotal
        -1.0, // lowtotal
        total_swap_mb,
        free_ram_mb,
        -1.0, // highfree
        -1.0, // lowfree
        free_swap_mb,
        -1.0, // memshared
        -1.0, // cached
        -1.0, // active
        -1.0, // bigfree
        -1.0, // buffers
        -1.0, // swapcached
        -1.0  // inactive
    )?;

    // 5. Disk I/O (Total aggregate disk stats represented as 'system')
    let has_any_logical_io = snapshot.filesystems.iter().any(|fs| fs.read_bps.is_some() || fs.write_bps.is_some());

    let (read_kb, write_kb) = if has_any_logical_io {
        let total_r: u64 = snapshot.filesystems.iter().map(|fs| fs.read_bps.unwrap_or(0)).sum();
        let total_w: u64 = snapshot.filesystems.iter().map(|fs| fs.write_bps.unwrap_or(0)).sum();
        (total_r as f64 / 1024.0, total_w as f64 / 1024.0)
    } else {
        (snapshot.disk_io.read_bps as f64 / 1024.0, snapshot.disk_io.write_bps as f64 / 1024.0)
    };
    let total_kb = read_kb + write_kb;
    // Calculate simple busy approximation (e.g. 10MB/s is 100% busy)
    let busy_pct = (total_kb / (1024.0 * 10.0) * 100.0).min(100.0);
    writeln!(w, "DISKBUSY,{},{:.1}", t_code, busy_pct)?;
    writeln!(w, "DISKREAD,{},{:.1}", t_code, read_kb)?;
    writeln!(w, "DISKWRITE,{},{:.1}", t_code, write_kb)?;
    writeln!(w, "DISKXFER,{},{:.1}", t_code, total_kb / 4.0)?; // estimated transfers
    writeln!(w, "DISKBSIZE,{},512.0", t_code)?; // default block size

    // 6. Network
    write!(w, "NET,{}", t_code)?;
    for net in &snapshot.networks {
        write!(w, ",{:.1}", net.rx_bytes_sec as f64 / 1024.0)?;
    }
    for net in &snapshot.networks {
        write!(w, ",{:.1}", net.tx_bytes_sec as f64 / 1024.0)?;
    }
    writeln!(w)?;

    write!(w, "NETPACKET,{}", t_code)?;
    for net in &snapshot.networks {
        write!(w, ",{:.1}", net.rx_bytes_sec as f64 / 1500.0)?; // estimated packets
    }
    for net in &snapshot.networks {
        write!(w, ",{:.1}", net.tx_bytes_sec as f64 / 1500.0)?;
    }
    writeln!(w)?;

    // 7. Filesystems
    write!(w, "JFSFILE,{}", t_code)?;
    for fs in &snapshot.filesystems {
        let total_gb = fs.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let free_gb = fs.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_pct = if total_gb > 0.0 {
            ((total_gb - free_gb) / total_gb) * 100.0
        } else {
            0.0
        };
        write!(w, ",{:.1}", used_pct)?;
    }
    writeln!(w)?;

    w.flush()?;
    Ok(())
}
