use clap::Args;

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// App to build in a multi-app workspace
    #[arg(long)]
    pub app: Option<String>,

    /// Environment section from origami.toml
    #[arg(long)]
    pub env: Option<String>,

    /// Locale to build for
    #[arg(long)]
    pub locale: Option<String>,

    /// Output directory
    #[arg(long)]
    pub out: Option<String>,
}

pub fn run(_args: BuildArgs) {
    unimplemented!("origami build — Block 07")
}
