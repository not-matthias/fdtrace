use crate::tracer::BpfTracer;
use args::Opt;
use structopt::StructOpt;

pub mod args;
pub mod syscall;
pub mod tracer;

// sudo -E cargo rr ../target/release/example
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
