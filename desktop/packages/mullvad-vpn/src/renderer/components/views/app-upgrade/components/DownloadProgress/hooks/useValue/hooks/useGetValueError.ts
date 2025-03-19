import { AppUpgradeError } from '../../../../../../../../../shared/daemon-rpc-types';
import { useAppUpgradeError } from '../../../../../hooks';
import { DOWNLOAD_COMPLETE_VALUE, FALLBACK_VALUE } from '../constants';

export const useGetValueError = () => {
  const appUpgradeError = useAppUpgradeError();

  const getValueError = () => {
    if (
      appUpgradeError === AppUpgradeError.startInstallerFailed ||
      appUpgradeError === AppUpgradeError.verificationFailed
    )
      return DOWNLOAD_COMPLETE_VALUE;

    return FALLBACK_VALUE;
  };

  return getValueError;
};
