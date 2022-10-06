enum CommandLineOptions {
  showChanges = '--show-changes',
  disableResetNavigation = '--disable-reset-navigation', // development only
  disableDevtoolsOpen = '--disable-devtools-open', // development only
  forwardRendererLog = '--forward-renderer-log', // development only
}

export const SHOULD_SHOW_CHANGES = process.argv.includes(CommandLineOptions.showChanges);
export const SHOULD_DISABLE_RESET_NAVIGATION = process.argv.includes(
  CommandLineOptions.disableResetNavigation,
);
export const SHOULD_DISABLE_DEVTOOLS_OPEN = process.argv.includes(
  CommandLineOptions.disableDevtoolsOpen,
);
export const SHOULD_FORWARD_RENDERER_LOG = process.argv.includes(
  CommandLineOptions.forwardRendererLog,
);
