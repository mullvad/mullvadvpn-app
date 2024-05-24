type Warning = 'launches-in-existing-process' | 'launches-elsewhere';

export interface IApplication {
  absolutepath: string;
  name: string;
  icon?: string;
}

export interface ISplitTunnelingApplication extends IApplication {
  deletable: boolean;
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

export interface ISplitTunnelingAppListRetriever {
  /**
   * Returns a list of all applications known to the app.
   */
  getApplications(
    updateCaches?: boolean,
  ): Promise<{ fromCache: boolean; applications: ISplitTunnelingApplication[] }>;

  /**
   * Returns an ISplitTunnelingApplication object containing the metadata related to the provided
   * application.
   */
  getMetadataForApplications(
    applicationPaths: string[],
  ): Promise<{ fromCache: boolean; applications: ISplitTunnelingApplication[] }>;

  /**
   * Resolves the actual executable path when an app is provided. On Windows this resolves links and
   * on macOS this finds the executable when an application bundle is provided.
   */
  resolveExecutablePath(providedPath: string): Promise<string>;

  /**
   * Adds an application to the internal cache.
   */
  addApplicationPathToCache(applicationPath: string): Promise<void>;

  /**
   * Removes an application from the internal cache.
   */
  removeApplicationFromCache(application: ISplitTunnelingApplication): void;
}
