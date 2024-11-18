use crate::tracer::BpfTracer;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fdtrace", about = "File syscall tracer")]
pub struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    pub debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,
}

pub mod syscall;
pub mod tracer;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Opt::from_args();

    // 1. Trace the target program
    //
    let tracer = BpfTracer::trace(&args.input).unwrap();
    if args.debug {
        tracer.debug_print();
    }

    // 2. Analyze the trace
    //

    Ok(())
}
