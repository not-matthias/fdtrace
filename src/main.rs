use crate::tracer::BpfTracer;
use analysis::Analysis;
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

pub mod analysis;
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
        tracer.print_to_file("debug.txt");
    }

    // 2. Analyze the trace
    //
    let analysis = Analysis::new(tracer.take_syscalls());
    analysis.print_result();

    Ok(())
}
