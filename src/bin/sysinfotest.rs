use nix::{errno::Errno, unistd::{geteuid, getresuid, getuid, seteuid}};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System, UpdateKind, Users};

const MEM_TO_MB: u64 = u64::pow(1024, 2);

fn main() -> Result<(), Errno> {
    change_euid(UIDTYPE::Real)?;
 
    let users: Users = Users::new_with_refreshed_list();

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

    let mut sys = System::new_with_specifics(specifics);
    sys.refresh_specifics(specifics);
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_specifics(specifics);
    println!("RAM used: {}%", 100*sys.used_memory() / sys.total_memory());
    println!("CPUs:");
    sys.cpus().iter().enumerate().for_each(|(i, cpu)| println!("\tCore {}: {: >5} at {}", i+1, format!("{:.1}", cpu.cpu_usage()), cpu.frequency()));

    println!("Num processes: {}", sys.processes().len());

    println!("Checking as real user...");
    exec_as_uid(UIDTYPE::Real, || print_mcprocesses(&mut sys, &users))?;
    println!("Checking as saved user...");
    exec_as_uid(UIDTYPE::Saved, || print_mcprocesses(&mut sys, &users))?;
    println!("...");
    
    Ok(())
}

fn print_mcprocesses(sys: &mut System, users: &Users) {
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();
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


enum UIDTYPE {
    Real,
    Saved
}

fn exec_as_uid<F>(to_type: UIDTYPE, f: F) -> Result<(), Errno> 
where
    F: FnOnce() -> ()
{
    let old_uid = geteuid();
    change_euid(to_type)?;
    f();
    seteuid(old_uid)?;
    Ok(())
}

fn change_euid(to_type: UIDTYPE) -> Result<(),Errno> {
    println!("{:->1$}", "", 80);
    let (real, effective, saved) = get_resuid_usernames();
    println!("Users before change:\n\treal:      {: >18}\n\tsaved:     {: >18}\n\teffective: {: >18}",
        real, saved, effective
    );

    match to_type {
        UIDTYPE::Real => {
            seteuid(getuid())?
        },
        UIDTYPE::Saved => {
            let res_uid = getresuid().unwrap();
            seteuid(res_uid.saved)?
        }
    }

    let (real, effective, saved) = get_resuid_usernames();
    println!("Users after change to {}:\n\treal:      {: >18}\n\tsaved:     {: >18}\n\teffective: {: >18}",
        match to_type { UIDTYPE::Real => "real", UIDTYPE::Saved => "saved" },
        real, saved, effective
    );

    println!("{:->1$}", "", 80);
    Ok(())
}

fn get_resuid_usernames() -> (String, String, String) {
    /*
    println!("UID: {:?}", Uid::current());
    println!("EUID: {:?}", Uid::effective());
    println!("R&E&S UID: {:?}", getresuid());
    */
    
    let res_uid = getresuid().unwrap();
    let real_user = nix::unistd::User::from_uid(res_uid.real).unwrap().unwrap();
    let saved_user = nix::unistd::User::from_uid(res_uid.saved).unwrap().unwrap();
    let effective_user = nix::unistd::User::from_uid(res_uid.effective).unwrap().unwrap();

    (real_user.name, effective_user.name, saved_user.name)
}