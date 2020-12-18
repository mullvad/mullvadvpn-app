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
  onlyShowIn?: string | string[];
  notShowIn?: string | string[];
  tryExec?: string;
}

export interface ILinuxSplitTunnelingApplication extends ILinuxApplication {
  warning?: Warning;
}
