mod args;
mod bpftrace;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // let target = Box::new(std::fs::File::create("trace.txt").expect("Can't create
    // file")); env_logger::Builder::new()
    //     .target(env_logger::Target::Pipe(target))
    //     .filter(None, log::LevelFilter::Info)
    //     .init();

    // let args = Opt::from_args();

    Ok(())
}
