use std::io::Read;

fn main() {
    // Session with wait but without read
    {
        let _file = std::fs::File::open("/etc/passwd").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // Session with wait and read
    {
        let mut file = std::fs::File::open("/etc/passwd").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
    }

    // Session with read
    let _ = std::fs::read_to_string("/etc/passwd").unwrap();
}
