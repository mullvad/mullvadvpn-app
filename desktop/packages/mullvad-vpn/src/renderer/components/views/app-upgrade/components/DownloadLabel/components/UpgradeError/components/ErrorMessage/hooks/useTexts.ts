import { messages } from '../../../../../../../../../../../shared/gettext';
import { useConnectionIsBlocked } from '../../../../../../../../../../redux/hooks';
import { useAppUpgradeError } from '../../../../../../../hooks';

export const useTexts = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeError = useAppUpgradeError();

  if (isBlocked) {
    return [
      // TRANSLATORS: Label displayed when an error occurred due to the connection being blocked
      messages.pgettext('app-upgrade-view', 'Connection blocked.'),
      // TRANSLATORS: Complimentary label displayed when an error occurred due to the connection being blocked
      messages.pgettext('app-upgrade-view', 'Try changing server or other settings.'),
    ];
  }

  switch (appUpgradeError) {
    case 'DOWNLOAD_FAILED':
      return [
        // TRANSLATORS: Label displayed when an error occurred due to the download failing
        messages.pgettext('app-upgrade-view', 'Download failed.'),
        // TRANSLATORS: Complimentary label displayed when an error occurred due to the download failing
        messages.pgettext(
          'app-upgrade-view',
          'Check your connection and/or firewall then try again. If this problem persists, please contact support.',
        ),
      ];
    case 'START_INSTALLER_FAILED':
      return [
        // TRANSLATORS: Label displayed when an error occurred due to the installer failing to start
        messages.pgettext('app-upgrade-view', 'Installer failed to start.'),
        // TRANSLATORS: Complimentary displayed when an error occurred due to the installer failing to start
        messages.pgettext(
          'app-upgrade-view',
          'Please try installing the update again. If this problem persists, please contact support.',
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
        messages.pgettext('app-upgrade-view', 'An unknown error occurred.'),
        // TRANSLATORS: Complimentary label displayed when an unknown error occurred
        messages.pgettext(
          'app-upgrade-view',
          'Please try again. If this problem persists, please contact support.',
        ),
      ];
  }
};
