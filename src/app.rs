pub use platform::App;

#[derive(Clone, Debug)]
pub enum AppEvent {
    DidFinishLaunching,
    WillTerminate,
    DidChangeScreenParameters,
    DoCommand(AppCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    ClearHistory,
    ToggleOptionDarkTheme,
}
