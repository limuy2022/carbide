#[derive(Clone, Debug, clap::Parser)]
#[command(long_about = r"
   O    O      Carbide is a Chromium based browser for the terminal.
    \  /       
O —— Cr —— O   In addition to the following options,
    /  \       Carbide also supports most Chromium options.
   O    O      ")]
pub struct CommandLine {
    #[arg(
        short,
        long,
        default_value = "60.0",
        help = "set the maximum number of frames per second"
    )]
    pub fps: f32,
    #[arg(
        short,
        long,
        default_value = "100.0",
        help = "set the zoom level in percent"
    )]
    pub zoom: f32,
    #[arg(short, long, default_value = "false", help = "render text as bitmaps")]
    pub bitmap: bool,
    #[arg(short, long)]
    pub shell_mode: bool,
    #[arg(short, long, default_value = "false")]
    pub version: bool,
}
