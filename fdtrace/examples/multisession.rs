use std::{io::Read, time::Duration};

fn main() {
    let mut file1 = std::fs::File::open("/etc/hosts").unwrap();
    let mut file2 = std::fs::File::open("/etc/passwd").unwrap();
    let mut file3 = std::fs::File::open("/etc/hostname").unwrap();

    let mut buf = String::new();

    let _ = file1.read_to_string(&mut buf);
    println!("{}", buf);

    let _ = file2.read_to_string(&mut buf);
    println!("{}", buf);

    let _ = file3.read_to_string(&mut buf);
    println!("{}", buf);

    std::thread::sleep(Duration::from_secs(1));
}
