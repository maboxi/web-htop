use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System, UpdateKind, Users};

fn main() {
    const MEM_TO_MB: u64 = u64::pow(1024, 2);

    let specifics = RefreshKind::new()
            .with_memory(MemoryRefreshKind::new().with_ram())
            .with_cpu(CpuRefreshKind::new().with_cpu_usage().with_frequency())
            .with_processes(ProcessRefreshKind::new()
                                .with_user(UpdateKind::OnlyIfNotSet)
                                .with_cmd(UpdateKind::OnlyIfNotSet)
                                .with_exe(UpdateKind::OnlyIfNotSet)
                                .with_cwd(UpdateKind::OnlyIfNotSet)
                                .with_memory()
                                .with_cpu()
                            );
    let users = Users::new_with_refreshed_list();

    let mut sys = System::new_with_specifics(specifics);
    sys.refresh_specifics(specifics);
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_specifics(specifics);
    println!("RAM used: {}%", 100*sys.used_memory() / sys.total_memory());
    println!("CPUs:");
    sys.cpus().iter().enumerate().for_each(|(i, cpu)| println!("\tCore {}: {: >5} at {}", i+1, format!("{:.1}", cpu.cpu_usage()), cpu.frequency()));

    println!("Num processes: {}", sys.processes().len());
    for (pid, process) in sys.processes().iter()
        .filter(|&(_, ref process)| process.cpu_usage() > 0.0)
        .filter(|&(_, ref process)| process.name().contains("java"))
        .filter(|&(_, ref process)| users.get_user_by_id(process.user_id().unwrap()).unwrap().name().contains("pufferpanel"))
    {
        println!("\t[{}] {}: {}", pid, users.get_user_by_id(process.user_id().unwrap()).unwrap().name(), process.name());
        println!("\t\tCMD: {}", process.cmd().join(" "));
        println!("\t\tEXE: {:?}", process.exe());
        println!("\t\tCWD: {:?}", process.cwd());
        println!("\t\tMemory: {}MB; virt: {}MB", process.memory() / MEM_TO_MB, process.virtual_memory() / MEM_TO_MB);
        println!("\t\tCPU: {}", process.cpu_usage());
    }
}