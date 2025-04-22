import { messages } from '../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../../../../redux/hooks';

export const useErrorMessage = () => {
  const { error } = useAppUpgradeError();

  switch (error) {
    case 'DOWNLOAD_FAILED':
      // TRANSLATORS: Description of an error which occurred in the app's upgrade process.
      // TRANSLATORS: when the download of the upgrade failed.
      // TRANSLATORS: The description is included in form field of a form
      // TRANSLATORS: a user can submit to report the error.
      return messages.pgettext('app-upgrade-view', 'Update downloader: Download failed.');
    case 'GENERAL_ERROR':
      // TRANSLATORS: Description of an error which occurred in the app's upgrade process
      // TRANSLATORS: when an unknown error occurred.
      // TRANSLATORS: The description is included in form field of a form
      // TRANSLATORS: a user can submit to report the error.
      return messages.pgettext('app-upgrade-view', 'Update downloader: An unknown error occurred.');
    case 'INSTALLER_FAILED':
      // TRANSLATORS: Description of an error which occurred in the app's upgrade process
      // TRANSLATORS: when the downloaded installer failed to install the upgrade successfully.
      // TRANSLATORS: The description is included in form field of a form
      // TRANSLATORS: a user can submit to report the error.
      return messages.pgettext(
        'app-upgrade-view',
        'Update downloader: The installer failed with an unknown error.',
      );
    case 'START_INSTALLER_FAILED':
      // TRANSLATORS: Description of an error which occurred in the app's upgrade process
      // TRANSLATORS: when the downloaded installer failed to start.
      // TRANSLATORS: The description is included in form field of a form
      // TRANSLATORS: a user can submit to report the error.
      return messages.pgettext(
        'app-upgrade-view',
        'Update downloader: The installer failed to start due to an unknown error.',
      );
    case 'VERIFICATION_FAILED':
      // TRANSLATORS: Description of an error which occurred in the app's upgrade process
      // TRANSLATORS: when the downloaded installer failed to verify its signature.
      // TRANSLATORS: The description is included in form field of a form
      // TRANSLATORS: a user can submit to report the error.
      return 'Update downloader: The installer failed verification.';
    default:
      return null;
  }
};
