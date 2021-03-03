mod networking;

fn mount_pseudo_filesystems() -> Result<(), nix::Error> {
    use nix::mount::{mount, MsFlags};
    mount::<str, str, str, str>(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), None)?;
    mount::<str, str, str, str>(Some("sysfs"), "/sys", Some("sysfs"), MsFlags::empty(), None)?;
    mount::<str, str, str, str>(
        Some("devtmpfs"),
        "/dev",
        Some("devtmpfs"),
        MsFlags::empty(),
        None,
    )
}

fn main() -> std::io::Result<()> {
    let start = std::time::Instant::now();
    let _ = mount_pseudo_filesystems();

    std::thread::spawn(move || {
        std::fs::File::open("/etc/hostname")
            .and_then(|file| {
                use std::io::prelude::*;
                let reader = std::io::BufReader::new(file);
                if let Some(Ok(hostname)) = reader.lines().next() {
                    let _ = nix::unistd::sethostname(hostname);
                }
                Ok(())
            })
            .unwrap_or(());
    });

    std::thread::spawn(move || {
        use nix::sys::socket::IpAddr;
        let _ = networking::setup(
            "eth0",
            IpAddr::new_v4(10, 0, 1, 10),
            IpAddr::new_v4(10, 0, 1, 255),
            IpAddr::new_v4(10, 0, 1, 2),
        );
    });

    let _ = std::process::Command::new("/usr/bin/python2.7")
        .arg("-c")
        .arg("print('Hello world')")
        .env("PYTHONHOME", "/")
        .spawn()?.wait()?;
    println!("{}", start.elapsed().as_millis());
    loop {
        use std::process::Command;
        println!("Welcome to Rustybox");
        if let Ok(_) = Command::new("/bin/getty")
            .arg("0")
            .arg("/dev/ttyS0")
            .output()
        {
            print!("{esc}[H{esc}[J", esc = 27 as char);
        } else {
            break;
        }
    }
    nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_POWER_OFF).expect("reboot");
    Ok(())
}
