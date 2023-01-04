import argvSplit from 'argv-split';
import child_process from 'child_process';
import path from 'path';

import { ILinuxSplitTunnelingApplication } from '../shared/application-types';
import { messages } from '../shared/gettext';
import { LaunchApplicationResult } from '../shared/ipc-schema';
import { Scheduler } from '../shared/scheduler';
import {
  DesktopEntry,
  findIconPath,
  getDesktopEntries,
  getImageDataUrl,
  readDesktopEntry,
  shouldShowApplication,
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

// Launches an application. The application parameter could be a path the an executable or .desktop
// file or an object representing an application
export async function launchApplication(
  app: ILinuxSplitTunnelingApplication | string,
): Promise<LaunchApplicationResult> {
  let excludeArguments: string[];
  try {
    excludeArguments = await getLaunchCommand(app);
  } catch (e) {
    const error = e as Error;
    return { error: error.message };
  }

  return new Promise((resolve, _reject) => {
    const scheduler = new Scheduler();
    const proc = child_process.spawn('mullvad-exclude', excludeArguments, { detached: true });

    // If the process exits within 200 milliseconds the user is notified that it failed to launch.
    scheduler.schedule(() => {
      proc.removeAllListeners();
      resolve({ success: true });
    }, 200);

    proc.stderr.on('data', (data) => {
      if (data.includes('Failed to launch the process') && data.includes('ENOENT')) {
        scheduler.cancel();
        proc.removeAllListeners();
        resolve({
          error:
            // TRANSLATORS: This error message is shown if the user tries to launch an app that
            // TRANSLATORS: doesn't exist.
            messages.pgettext('split-tunneling-view', 'Please try again or send a problem report.'),
        });
      }
    });
    proc.once('exit', (code) => {
      scheduler.cancel();
      proc.removeAllListeners();

      if (code === 1) {
        resolve({
          error:
            // TRANSLATORS: This error message is shown if an application failes during startup.
            messages.pgettext('split-tunneling-view', 'Please try again or send a problem report.'),
        });
      } else {
        resolve({ success: true });
      }
    });
  });
}

// Takes the same argument as launchApplication and returns the command to run
async function getLaunchCommand(app: ILinuxSplitTunnelingApplication | string): Promise<string[]> {
  if (typeof app === 'object') {
    return formatExec(app.exec);
  } else if (path.extname(app) === '.desktop') {
    const entry = await readDesktopEntry(app);
    if (entry.exec !== undefined) {
      return formatExec(entry.exec);
    } else {
      throw new Error(
        // TRANSLATORS: This error message is shown if the user tries to launch a Linux desktop
        // TRANSLATORS: entry file that doesn't contain the required 'Exec' value.
        messages.pgettext('split-tunneling-view', 'Please send a problem report.'),
      );
    }
  } else {
    return [app];
  }
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
