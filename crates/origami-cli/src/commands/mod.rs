pub mod build;
pub mod check;
pub mod dev;
pub mod init;
pub mod test;
pub mod unsafe_report;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the development server
    Dev(dev::DevArgs),
    /// Build the application for production
    Build(build::BuildArgs),
    /// Validate the project without generating output
    Check(check::CheckArgs),
    /// Run tests and visual preview
    Test(test::TestArgs),
    /// Initialise a new Origami project
    Init(init::InitArgs),
    /// Report all unsafe design system overrides
    UnsafeReport(unsafe_report::UnsafeReportArgs),
}
