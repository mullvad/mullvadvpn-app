import { useShouldAppUpgradeInstallManually } from '../../../../hooks';

export const useShowInstallButton = () => {
  const shouldAppUpgradeInstallManually = useShouldAppUpgradeInstallManually();

  const showInstallButton = shouldAppUpgradeInstallManually;

  return showInstallButton;
};
