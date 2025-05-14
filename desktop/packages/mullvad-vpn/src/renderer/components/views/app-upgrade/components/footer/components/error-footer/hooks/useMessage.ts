import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../../../../../../redux/hooks';

export const useMessage = () => {
  const { error } = useAppUpgradeError();

  switch (error) {
    case 'DOWNLOAD_FAILED':
      // TRANSLATORS: Label displayed when an error occurred due to the download failing
      return messages.pgettext(
        'app-upgrade-view',
        'Download failed, please check your connection/firewall and try again, or send a problem report.',
      );
    case 'INSTALLER_FAILED':
      // TRANSLATORS: Label displayed when an error occurred within the installer
      return messages.pgettext(
        'app-upgrade-view',
        'Installer quit unexpectedly, please try again or send a problem report.',
      );
    case 'START_INSTALLER_FAILED':
      // TRANSLATORS: Label displayed when an error occurred due to the installer failing to start
      // TRANSLATORS: and the suggested resolution is to download the update again.
      return messages.pgettext(
        'app-upgrade-view',
        'Could not open installer, please try again or send a problem report.',
      );
    case 'VERIFICATION_FAILED':
      // TRANSLATORS: Label displayed when an error occurred due to the installer failed verification
      return messages.pgettext(
        'app-upgrade-view',
        'Verification failed, please try again or send a problem report.',
      );
    default:
      // TRANSLATORS: Label displayed when an unknown error occurred
      return messages.pgettext(
        'app-upgrade-view',
        'Unknown error occurred. Please try again or send a problem report.',
      );
  }
};
