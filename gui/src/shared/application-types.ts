type Warning = 'launches-in-existing-process' | 'launches-elsewhere';

export interface IApplication {
  absolutepath: string;
  name: string;
  icon?: string;
}

export interface ILinuxApplication extends IApplication {
  exec: string;
  type: string;
  terminal?: string;
  noDisplay?: string;
  hidden?: string;
  onlyShowIn?: string[];
  notShowIn?: string[];
  tryExec?: string;
}

export interface ILinuxSplitTunnelingApplication extends ILinuxApplication {
  warning?: Warning;
}
