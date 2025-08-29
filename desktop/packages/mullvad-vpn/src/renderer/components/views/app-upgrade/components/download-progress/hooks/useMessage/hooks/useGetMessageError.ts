import { useHasAppUpgradeVerifiedInstallerPath } from '../../../../../../../../hooks';
import { useAppUpgradeError } from '../../../../../../../../redux/hooks';
import { translations } from '../constants';

export const useGetMessageError = () => {
  const { error } = useAppUpgradeError();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();

  const getMessageError = () => {
    if (error) {
      switch (error) {
        case 'DOWNLOAD_FAILED':
          return translations.downloadFailed;
        case 'INSTALLER_FAILED':
        case 'START_INSTALLER_FAILED':
        case 'VERIFICATION_FAILED':
          return translations.downloadComplete;
        case 'GENERAL_ERROR':
          return hasAppUpgradeVerifiedInstallerPath ? translations.downloadComplete : null;
        default:
          return error satisfies never;
      }
    }

    return null;
  };

  return getMessageError;
};
