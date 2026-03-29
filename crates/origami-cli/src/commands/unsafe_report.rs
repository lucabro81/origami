use clap::Args;

#[derive(Args, Debug)]
pub struct UnsafeReportArgs {
    /// App to report on in a multi-app workspace
    #[arg(long)]
    pub app: Option<String>,

    /// Output format: text (default) or json
    #[arg(long, default_value = "text")]
    pub format: String,
}

pub fn run(_args: UnsafeReportArgs) {
    unimplemented!("origami unsafe-report — Block 07")
}
