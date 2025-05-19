import { useAppUpgradeError } from '../../../../../../../../redux/hooks';
import { translations } from '../constants';

export const useGetMessageError = () => {
  const { error } = useAppUpgradeError();

  const getMessageError = () => {
    switch (error) {
      case 'DOWNLOAD_FAILED':
        return translations.downloadFailed;
      case 'INSTALLER_FAILED':
      case 'START_INSTALLER_FAILED':
      case 'VERIFICATION_FAILED':
        return translations.downloadComplete;
      default:
        return null;
    }
  };

  return getMessageError;
};
