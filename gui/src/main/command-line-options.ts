class CommandLineOption {
  private options: string[];

  public constructor(private description: string, ...options: string[]) {
    this.options = options;
  }

  public get match(): boolean {
    return this.options.some((option) => process.argv.includes(option));
  }

  public format(): string {
    return formatOption(this.description, ...this.options);
  }
}

class DevelopmentCommandLineOption extends CommandLineOption {
  public constructor(...options: string[]) {
    super('', ...options);
  }

  public get match(): boolean {
    return process.env.NODE_ENV === 'development' && super.match;
  }
}

export const CommandLineOptions = {
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

function formatOption(description: string, ...options: string[]) {
  const joinedOptions = options.join(', ');
  const padding = '                    ';
  const paddedOptions = (joinedOptions + padding).slice(0, -joinedOptions.length);
  return '    ' + paddedOptions + description;
}
