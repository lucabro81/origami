use clap::Args;

#[derive(Args, Debug)]
pub struct DevArgs {
    /// App to run in a multi-app workspace
    #[arg(long)]
    pub app: Option<String>,

    /// Environment section from origami.toml (default: dev)
    #[arg(long)]
    pub env: Option<String>,

    /// Override local server port
    #[arg(long)]
    pub port: Option<u16>,

    /// Bind address (default: localhost)
    #[arg(long)]
    pub host: Option<String>,
}

pub fn run(_args: DevArgs) {
    unimplemented!("origami dev — Block 03")
}
