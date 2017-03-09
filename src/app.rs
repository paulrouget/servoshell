pub use platform::App;
use commands::AppCommand;

#[derive(Clone, Debug)]
pub enum AppEvent {
    DidFinishLaunching,
    WillTerminate,
    DidChangeScreenParameters,
    DoCommand(AppCommand),
}
