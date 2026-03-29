use clap::Args;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Name of the new project
    pub project_name: String,

    /// Name of the default app (default: web)
    #[arg(long)]
    pub app: Option<String>,

    /// Skip generating example pages and components
    #[arg(long)]
    pub no_example: bool,
}

pub fn run(_args: InitArgs) {
    unimplemented!("origami init — Block 07")
}
