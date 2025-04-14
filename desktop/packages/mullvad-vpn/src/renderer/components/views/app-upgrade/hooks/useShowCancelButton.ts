import { useIsAppUpgradePreparing } from '../../../../hooks';

export const useShowCancelButton = () => {
  const isAppUpgradePreparing = useIsAppUpgradePreparing();

  const showCancelButton = isAppUpgradePreparing;

  return showCancelButton;
};
