use clap::Args;

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// App to check in a multi-app workspace
    #[arg(long)]
    pub app: Option<String>,
}

pub fn run(_args: CheckArgs) {
    unimplemented!("origami check — Block 01")
}
