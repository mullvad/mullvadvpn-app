import argvSplit from 'argv-split';
import child_process from 'child_process';
import path from 'path';
import { ILinuxSplitTunnelingApplication } from '../shared/application-types';
import {
  getDesktopEntries,
  readDesktopEntry,
  findIconPath,
  getImageDataUrl,
  shouldShowApplication,
  DesktopEntry,
} from './linux-desktop-entry';

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

export async function getApplications(locale: string): Promise<ILinuxSplitTunnelingApplication[]> {
  const desktopEntryPaths = await getDesktopEntries();
  const desktopEntries: DesktopEntry[] = [];

  for (const entryPath of desktopEntryPaths) {
    try {
      desktopEntries.push(await readDesktopEntry(entryPath, locale));
    } catch (e) {
      // no-op
    }
  }

  const applications = desktopEntries
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
    if (app.icon === undefined) {
      return app;
    }

    const iconPath = path.isAbsolute(app.icon) ? app.icon : await findIconPath(app.icon);
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
