import { NativeImage, nativeImage } from 'electron';
import fs from 'fs/promises';
import { userInfo } from 'os';
import path from 'path';
import plist from 'simple-plist';
import { promisify } from 'util';

import {
  ISplitTunnelingApplication,
  ISplitTunnelingAppListRetriever,
} from '../shared/application-types';
import log from '../shared/logging';

const readPlist = promisify(plist.readFile);

type Plist = Record<string, unknown>;

export class MacOsSplitTunnelingAppListRetriever implements ISplitTunnelingAppListRetriever {
  /**
   * Cache of all previously scanned applications.
   */
  private applicationCache = new ApplicationCache();
  /**
   * List of apps that have been added manually by the user.
   */
  private additionalApplications = new AdditionalApplications();

  public async getApplications(updateCaches = false): Promise<{
    fromCache: boolean;
    applications: ISplitTunnelingApplication[];
  }> {
    const fromCache = !updateCaches && !this.applicationCache.isEmpty();

    // Update cache if requested or if cache is empty.
    if (!fromCache) {
      const applicationBundlePaths = await this.findApplicationBundlePaths();
      const executablePaths = this.additionalApplications.values();
      await Promise.all([
        // `getApplication updates the cache so no need to use the result.`
        ...applicationBundlePaths.map((applicationBundlePath) =>
          this.getApplication(applicationBundlePath, false),
        ),
        ...executablePaths.map((executablePath) => this.getApplication(executablePath, true)),
      ]);
    }

    // Return applications from cache.
    return { fromCache, applications: this.applicationCache.values() };
  }

  public async getMetadataForApplications(
    applicationPaths: string[],
  ): Promise<{ fromCache: boolean; applications: ISplitTunnelingApplication[] }> {
    await Promise.all(
      applicationPaths
        .filter((applicationPath) => !this.applicationCache.includes(applicationPath))
        .map((applicationPath) => this.addApplicationPathToCache(applicationPath)),
    );

    const applications = await this.getApplications();

    applications.applications = applications.applications.filter((application) =>
      applicationPaths.some(
        (applicationPath) =>
          applicationPath.toLowerCase() === application.absolutepath.toLowerCase(),
      ),
    );

    return applications;
  }

  public removeApplicationFromCache(application: ISplitTunnelingApplication): void {
    this.applicationCache.remove(application);
    this.additionalApplications.remove(application);
  }

  public async addApplicationPathToCache(applicationPath: string): Promise<void> {
    const application = await this.getApplication(applicationPath, true);
    if (application?.deletable) {
      this.additionalApplications.add(application);
    }
  }

  public async resolveExecutablePath(applicationPath: string): Promise<string> {
    if (path.extname(applicationPath) === '.app') {
      const macOsApplication = await this.createMacOsApplication(applicationPath, false);
      return macOsApplication?.absolutepath ?? applicationPath;
    } else {
      return applicationPath;
    }
  }

  /**
   * Creates an `ISplitTunnelingApplication` and adds it to the cache.
   */
  private async getApplication(
    applicationPath: string,
    deletable: boolean,
  ): Promise<ISplitTunnelingApplication | undefined> {
    const application = await this.createApplication(applicationPath, deletable);

    if (application !== undefined) {
      this.applicationCache.add(application);
    }

    return application;
  }

  private async findApplicationBundlePaths() {
    const readdirPromises = this.getAppDirectories().map((directory) =>
      this.readDirectory(directory),
    );
    const applicationBundlePaths = (await Promise.all(readdirPromises)).flat();
    return applicationBundlePaths.filter((filePath) => {
      const parsedFilePath = path.parse(filePath);
      return (
        parsedFilePath.ext === '.app' &&
        !parsedFilePath.name.startsWith('.') &&
        parsedFilePath.name !== 'Mullvad VPN'
      );
    });
  }

  /**
   * Returns contents of directory with results as absolute paths.
   */
  private async readDirectory(applicationDir: string) {
    try {
      const basenames = await fs.readdir(applicationDir);
      return basenames.map((basename) => path.join(applicationDir, basename));
    } catch (err) {
      const e = err as NodeJS.ErrnoException;
      if (e.code !== 'ENOENT' && e.code !== 'ENOTDIR') {
        log.error(`Failed to read directory contents: ${applicationDir}`, err);
      }
      return [];
    }
  }

  private async readApplicationBundlePlist(applicationBundlePath: string): Promise<Plist> {
    const plistPath = path.join(applicationBundlePath, 'Contents', 'Info.plist');
    return (await readPlist(plistPath)) ?? {};
  }

  /**
   * Creates an `ISplitTunnelingApplication` for any type of application.
   */
  private async createApplication(
    applicationPath: string,
    deletable: boolean,
  ): Promise<ISplitTunnelingApplication | undefined> {
    if (path.extname(applicationPath) === '.app') {
      return this.createMacOsApplication(applicationPath, deletable);
    }

    const applicationDirectory = this.getApplicationDirectoryForExecutable(applicationPath);
    if (applicationDirectory) {
      const additionalApplication = await this.createMacOsApplication(
        applicationDirectory,
        deletable,
      );
      if (additionalApplication?.absolutepath === applicationPath) {
        return additionalApplication;
      }
    }

    return this.createExecutableApplication(applicationPath, deletable);
  }

  /**
   * Creates an `ISplitTunnelingApplication` for the provided executable.
   */
  private async createExecutableApplication(
    executablePath: string,
    deletable: boolean,
  ): Promise<ISplitTunnelingApplication> {
    return {
      absolutepath: executablePath,
      name: path.basename(executablePath),
      icon: (await this.getApplicationIcon(executablePath)).toDataURL(),
      deletable,
    };
  }

  /**
   * Creates an `ISplitTunnelingApplication` for the provided application bundle.
   */
  private async createMacOsApplication(
    applicationBundlePath: string,
    deletable: boolean,
  ): Promise<ISplitTunnelingApplication | undefined> {
    const appInfo = await this.readApplicationBundlePlist(applicationBundlePath);

    if (!('CFBundleExecutable' in appInfo) || typeof appInfo.CFBundleExecutable !== 'string') {
      return undefined;
    }

    const name = this.getApplicationName(appInfo);
    if (!name) {
      return undefined;
    }

    const icon = await this.getApplicationIcon(applicationBundlePath);
    const executablePath = path.join(
      applicationBundlePath,
      'Contents',
      'MacOS',
      appInfo.CFBundleExecutable,
    );

    return {
      absolutepath: executablePath,
      name,
      icon: icon.toDataURL(),
      deletable,
    };
  }

  private getApplicationName(appInfo: Plist): string | void {
    if ('CFBundleDisplayName' in appInfo && typeof appInfo.CFBundleDisplayName === 'string') {
      return appInfo.CFBundleDisplayName;
    }

    if ('CFBundleName' in appInfo && typeof appInfo.CFBundleName === 'string') {
      return appInfo.CFBundleName;
    }
  }

  private async getApplicationIcon(applicationPath: string): Promise<NativeImage> {
    const applicationDirectory =
      this.getApplicationDirectoryForExecutable(applicationPath) ?? applicationPath;

    try {
      // 70x70 is the size at which the icon will be rendered in the split tunneling view accounting
      // for HiDPI displays.
      return await nativeImage.createThumbnailFromPath(applicationDirectory, {
        height: 70,
        width: 70,
      });
    } catch {
      log.info('Failed to fetch icon for split tunneling application:', applicationPath);
      return nativeImage.createEmpty();
    }
  }

  /**
   * Returns path to the application bundle if the provided path is or is part of an application
   * bundle.
   */
  private getApplicationDirectoryForExecutable(currentPath: string): string | undefined {
    const parsedPath = path.parse(currentPath);
    if (parsedPath.ext === '.app') {
      return currentPath;
    } else if (parsedPath.dir === '/') {
      return undefined;
    } else {
      return this.getApplicationDirectoryForExecutable(parsedPath.dir);
    }
  }

  /**
   * Returns the directories to be scanned for application bundles.
   */
  private getAppDirectories() {
    return [
      '/Applications',
      '/Applications/Utilities',
      '/System/Applications',
      path.join('/', 'Users', userInfo().username, 'Applications'),
    ];
  }
}

/**
 * Cache of all previously scanned applications.
 */
class ApplicationCache {
  private cache: Record<string, ISplitTunnelingApplication> = {};

  public add(application: ISplitTunnelingApplication) {
    const cacheKey = application.absolutepath.toLowerCase();
    this.cache[cacheKey] = this.merge(application, this.cache[cacheKey]);
  }

  public remove(application: ISplitTunnelingApplication) {
    delete this.cache[application.absolutepath.toLowerCase()];
  }

  public values(): Array<ISplitTunnelingApplication> {
    return Object.values(this.cache);
  }

  public isEmpty(): boolean {
    return Object.keys(this.cache).length === 0;
  }

  public includes(application: ISplitTunnelingApplication | string) {
    const cacheKey = typeof application === 'string' ? application : application.absolutepath;
    return this.cache[cacheKey.toLowerCase()] !== undefined;
  }

  /**
   * Merges two applications by using the values from the new one but respects the `deletable`
   * property on the old one.
   */
  private merge(
    newApplication: ISplitTunnelingApplication,
    oldApplication?: ISplitTunnelingApplication,
  ): ISplitTunnelingApplication {
    if (oldApplication === undefined) {
      return newApplication;
    }

    newApplication.deletable =
      newApplication.deletable === true && oldApplication.deletable === true;
    return newApplication;
  }
}

/**
 * List of apps that have been added manually by the user.
 */
class AdditionalApplications {
  private executablePaths: Record<string, string> = {};

  public add(application: ISplitTunnelingApplication | string) {
    const executablePath = typeof application === 'string' ? application : application.absolutepath;
    this.executablePaths[executablePath.toLowerCase()] = executablePath;
  }

  public remove(application: ISplitTunnelingApplication | string) {
    const executablePath = typeof application === 'string' ? application : application.absolutepath;
    delete this.executablePaths[executablePath.toLowerCase()];
  }

  public values(): Array<string> {
    return Object.values(this.executablePaths);
  }
}
