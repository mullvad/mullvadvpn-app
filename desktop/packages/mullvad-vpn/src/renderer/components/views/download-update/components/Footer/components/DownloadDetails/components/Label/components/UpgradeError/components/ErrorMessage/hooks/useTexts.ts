import { AppUpgradeError } from '../../../../../../../../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../../../../../../../../shared/gettext';
import { useAppUpgradeEvent } from '../../../../../../../../../../../hooks';
import { useIsBlocked } from '../../../../../../../../../hooks';

export const useTexts = () => {
  const isBlocked = useIsBlocked();
  const appUpgradeEvent = useAppUpgradeEvent();

  if (isBlocked) {
    return [
      // TRANSLATORS: Label displayed when an error occurred due to the connection being blocked
      messages.pgettext('download-update-view', 'Connection blocked.'),
      // TRANSLATORS: Complimentary label displayed when an error occurred due to the connection being blocked
      messages.pgettext('download-update-view', 'Try changing server or other settings.'),
    ];
  }

  if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_ERROR') {
    switch (appUpgradeEvent.error) {
      case AppUpgradeError.downloadFailed:
        return [
          // TRANSLATORS: Label displayed when an error occurred due to the download failing
          messages.pgettext('download-update-view', 'Download failed.'),
          // TRANSLATORS: Complimentary label displayed when an error occurred due to the download failing
          messages.pgettext(
            'download-update-view',
            'Check your connection and/or firewall then try again. If this problem persists, please contact support.',
          ),
        ];
      case AppUpgradeError.startInstallerFailed:
        return [
          // TRANSLATORS: Label displayed when an error occurred due to the installer failing to start
          messages.pgettext('download-update-view', 'Connection blocked.'),
          // TRANSLATORS: Complimentary displayed when an error occurred due to the installer failing to start
          messages.pgettext('download-update-view', 'Try changing server or other settings'),
        ];
      case AppUpgradeError.verificationFailed:
        return [
          // TRANSLATORS: Label displayed when an error occurred due to the installer failed verification
          messages.pgettext(
            'download-update-view',
            'Verification failed. Try again. If this problem persists, please contact support.',
          ),
        ];
      default:
        break;
    }
  }

  return [
    // TRANSLATORS: Label displayed when an unknown error occurred
    messages.pgettext('download-update-view', 'An unknown error occurred.'),
    // TRANSLATORS: Complimentary label displayed when an unknown error occurred
    messages.pgettext(
      'download-update-view',
      'Please try again. If this problem persists, please contact support.',
    ),
  ];
};
