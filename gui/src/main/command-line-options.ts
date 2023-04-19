class CommandLineOption {
  private flags: string[];

  public constructor(private description: string, ...flags: string[]) {
    this.flags = flags;
  }

  public get match(): boolean {
    return this.flags.some((flag) => process.argv.includes(flag));
  }

  public format(): string {
    return formatOption(this.description, ...this.flags);
  }
}

class DevelopmentCommandLineOption extends CommandLineOption {
  public constructor(...flags: string[]) {
    super('', ...flags);
  }

  public get match(): boolean {
    return process.env.NODE_ENV === 'development' && super.match;
  }
}

export const CommandLineOptions = {
  help: new CommandLineOption('Print this help text', '--help', '-h'),
  version: new CommandLineOption('Print the app version', '--version'),
  showChanges: new CommandLineOption('Show changes dialog', '--show-changes'),
  disableResetNavigation: new DevelopmentCommandLineOption('--disable-reset-navigation'),
  disableDevtoolsOpen: new DevelopmentCommandLineOption('--disable-devtools-open'),
  forwardRendererLog: new DevelopmentCommandLineOption('--forward-renderer-log'),
} as const;

export function printCommandLineOptions() {
  Object.values(CommandLineOptions).forEach((option) => {
    if (!(option instanceof DevelopmentCommandLineOption)) {
      console.log(option.format());
    }
  });
}

export function printElectronOptions() {
  console.log(formatOption('Run without renderer process sandboxed', '--no-sandbox'));
  console.log(formatOption('Run without hardware acceleration for graphics', '--disable-gpu'));
}

// This functions format options into one line, e.g.
//     --help              Print this help text
// The line starts with 4 spaces and the flags and description are separated with spaces to align
// the descriptions
function formatOption(description: string, ...flags: string[]) {
  const joinedFlags = flags.join(', ');
  const padding = '                    ';
  const paddedFlags = (joinedFlags + padding).slice(0, -joinedFlags.length);
  return '    ' + paddedFlags + description;
}
