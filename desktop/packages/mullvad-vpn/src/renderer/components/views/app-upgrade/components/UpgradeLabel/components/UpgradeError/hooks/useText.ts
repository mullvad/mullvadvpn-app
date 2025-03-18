import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeError, useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useText = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const { appUpgradeError } = useAppUpgradeError();

  if (isBlocked) {
    return [
      // TRANSLATORS: Label displayed when an error occurred due to the connection being blocked
      messages.pgettext(
        'app-upgrade-view',
        'Connection blocked. Try changing server or other settings',
      ),
    ];
  }

  switch (appUpgradeError) {
    case 'DOWNLOAD_FAILED':
      return [
        // TRANSLATORS: Label displayed when an error occurred due to the download failing
        messages.pgettext(
          'app-upgrade-view',
          'Unable to download update. Check your connection and/or firewall then try again. If this problem persists, please contact support.',
        ),
      ];
    case 'START_INSTALLER_AUTOMATIC_FAILED':
    case 'START_INSTALLER_FAILED':
      return [
        // TRANSLATORS: Label displayed when an error occurred due to the installer failing to start
        // TRANSLATORS: and the suggested resolution is to download the update again.
        messages.pgettext(
          'app-upgrade-view',
          'Could not start the update installer, try downloading it again. If this problem persists, please contact support.',
        ),
      ];
    case 'VERIFICATION_FAILED':
      return [
        // TRANSLATORS: Label displayed when an error occurred due to the installer failed verification
        messages.pgettext(
          'app-upgrade-view',
          'Verification failed. Try again. If this problem persists, please contact support.',
        ),
      ];
    default:
      return [
        // TRANSLATORS: Label displayed when an unknown error occurred
        messages.pgettext(
          'app-upgrade-view',
          'An unknown error occurred. Please try again. If this problem persists, please contact support.',
        ),
      ];
  }
};
