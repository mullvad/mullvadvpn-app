import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../../../../../../redux/hooks';
import { useErrorCountExceeded } from '../../../../../hooks';

export const useMessage = () => {
  const { error } = useAppUpgradeError();
  const errorCountExceeded = useErrorCountExceeded();

  if (errorCountExceeded) {
    // TRANSLATORS: Label displayed when an error occurred due to the download failing
    return messages.pgettext(
      'app-upgrade-view',
      'Having problems? Try downloading the update from our website. If this problem persists, please contact support.',
    );
  }

  switch (error) {
    case 'DOWNLOAD_FAILED':
      // TRANSLATORS: Label displayed when an error occurred due to the download failing
      return messages.pgettext(
        'app-upgrade-view',
        'Unable to download update. Check your connection and/or firewall then try again. If this problem persists, please contact support.',
      );
    case 'INSTALLER_FAILED':
      // TRANSLATORS: Label displayed when an error occurred within the installer
      return messages.pgettext(
        'app-upgrade-view',
        'Installer encountered an error. Try again. If this problem persists, please contact support.',
      );
    case 'START_INSTALLER_AUTOMATIC_FAILED':
    case 'START_INSTALLER_FAILED':
      // TRANSLATORS: Label displayed when an error occurred due to the installer failing to start
      // TRANSLATORS: and the suggested resolution is to download the update again.
      return messages.pgettext(
        'app-upgrade-view',
        'Could not start the update installer, try downloading it again. If this problem persists, please contact support.',
      );
    case 'VERIFICATION_FAILED':
      // TRANSLATORS: Label displayed when an error occurred due to the installer failed verification
      return messages.pgettext(
        'app-upgrade-view',
        'Verification failed. Try again. If this problem persists, please contact support.',
      );
    default:
      // TRANSLATORS: Label displayed when an unknown error occurred
      return messages.pgettext(
        'app-upgrade-view',
        'An unknown error occurred. Please try again. If this problem persists, please contact support.',
      );
  }
};
