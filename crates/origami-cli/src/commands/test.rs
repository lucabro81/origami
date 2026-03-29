use clap::Args;

#[derive(Args, Debug)]
pub struct TestArgs {
    /// Build and open the visual preview app
    #[arg(long)]
    pub preview: bool,

    /// Output a static preview site
    #[arg(long)]
    pub build_preview: bool,

    /// Run snapshot tests
    #[arg(long)]
    pub snapshot: bool,

    /// Update recorded snapshots
    #[arg(long)]
    pub update_snapshots: bool,

    /// Run E2E tests
    #[arg(long)]
    pub e2e: bool,

    /// Run accessibility checks
    #[arg(long)]
    pub a11y: bool,

    /// Filter tests by name
    #[arg(long)]
    pub filter: Option<String>,

    /// Environment section from origami.toml
    #[arg(long)]
    pub env: Option<String>,

    /// Watch mode: re-run on file change
    #[arg(long)]
    pub watch: bool,
}

pub fn run(_args: TestArgs) {
    unimplemented!("origami test — Block 05")
}
