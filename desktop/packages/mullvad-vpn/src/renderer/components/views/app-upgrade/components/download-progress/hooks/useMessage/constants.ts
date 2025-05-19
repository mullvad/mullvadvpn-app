import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';

export const translations = {
  downloadComplete:
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
    messages.pgettext('app-upgrade-view', 'Download complete!'),
  downloadFailed:
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update fails
    messages.pgettext('app-upgrade-view', 'Download failed'),
  downloadFewSecondsRemaining:
    // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
    // TRANSLATORS: with the estimated time of completion is within a few seconds.
    messages.pgettext('app-upgrade-view', 'A few seconds remaining...'),
  downloadPaused:
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update has been paused
    messages.pgettext('app-upgrade-view', 'Download paused'),
  downloadStarting:
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update is starting
    messages.pgettext('app-upgrade-view', 'Starting download...'),
  getDownloadMinutesRemaining: (minutes: number) =>
    sprintf(
      // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
      // TRANSLATORS: with the estimated time of completion represented in minutes.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(minutes)s - Will be replaced with remaining minutes until download is complete
      messages.pgettext('app-upgrade-view', 'About %(minutes)s minutes remaining...'),
      {
        minutes,
      },
    ),
  getDownloadSecondsRemaining: (seconds: number) =>
    sprintf(
      // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
      // TRANSLATORS: with the estimated time of completion represented in seconds.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(second)s - Will be replaced with remaining seconds until download is complete
      messages.pgettext('app-upgrade-view', 'About %(seconds)s seconds remaining...'),
      {
        seconds,
      },
    ),
};
