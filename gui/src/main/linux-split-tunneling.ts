import argvSplit from 'argv-split';
import child_process from 'child_process';
import linuxAppList, { AppData } from 'linux-app-list';
import path from 'path';
import { pascalCaseToCamelCase } from './transform-object-keys';
import { ILinuxSplitTunnelingApplication } from '../shared/application-types';
import { findIconPath, getImageDataUrl, shouldShowApplication } from './linux-desktop-entry';

const PROBLEMATIC_APPLICATIONS = {
  launchingInExistingProcess: [
    'brave-browser-stable',
    'chromium-browser',
    'firefox',
    'firefox-esr',
    'google-chrome-stable',
    'mate-terminal',
    'opera',
    'xfce4-terminal',
  ],
  launchingElsewhere: ['gnome-terminal'],
};

export function launchApplication(app: ILinuxSplitTunnelingApplication | string) {
  const excludeArguments = typeof app === 'string' ? [app] : formatExec(app.exec);
  child_process.spawn('mullvad-exclude', excludeArguments, { detached: true });
}

// Removes placeholder arguments and separates command into list of strings
function formatExec(exec: string) {
  return argvSplit(exec).filter((argument: string) => !/%[cdDfFikmnNuUv]/.test(argument));
}

// TODO: Switch to asyncronous reading of .desktop files
export function getApplications(locale: string): Promise<ILinuxSplitTunnelingApplication[]> {
  const appList = linuxAppList();
  const applications = appList
    .list()
    .map((filename) => {
      const applications = localizeNameAndIcon(appList.data(filename), locale);
      return pascalCaseToCamelCase<ILinuxSplitTunnelingApplication>(applications);
    })
    .filter(shouldShowApplication)
    .map(addApplicationWarnings)
    .sort((a, b) => a.name.localeCompare(b.name))
    .map(replaceIconNameWithDataUrl);

  return Promise.all(applications);
}

async function replaceIconNameWithDataUrl(
  app: ILinuxSplitTunnelingApplication,
): Promise<ILinuxSplitTunnelingApplication> {
  try {
    // Either the app has no icon or it's already an absolute path.
    if (app.icon === undefined || path.isAbsolute(app.icon)) {
      return app;
    }

    const iconPath = await findIconPath(app.icon);
    if (iconPath === undefined) {
      return app;
    }

    return { ...app, icon: await getImageDataUrl(iconPath) };
  } catch (e) {
    return app;
  }
}

function addApplicationWarnings(
  application: ILinuxSplitTunnelingApplication,
): ILinuxSplitTunnelingApplication {
  const binaryBasename = path.basename(application.exec!.split(' ')[0]);
  if (PROBLEMATIC_APPLICATIONS.launchingInExistingProcess.includes(binaryBasename)) {
    return {
      ...application,
      warning: 'launches-in-existing-process',
    };
  } else if (PROBLEMATIC_APPLICATIONS.launchingElsewhere.includes(binaryBasename)) {
    return {
      ...application,
      warning: 'launches-elsewhere',
    };
  } else {
    return application;
  }
}

function localizeNameAndIcon(application: AppData, locale: string) {
  if (application.lang) {
    // linux-app-list prefixes and suffixes the locale keys with a space for some reason.
    const lang = Object.fromEntries(
      Object.entries(application.lang).map(([locale, value]) => [locale.trim(), value]),
    )[locale];

    application.Name = lang?.Name ?? application.Name;
    application.Icon = lang?.Icon ?? application.Icon;
  }

  return application;
}
