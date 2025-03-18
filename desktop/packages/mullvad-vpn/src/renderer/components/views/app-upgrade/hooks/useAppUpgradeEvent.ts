import { useSelector } from '../../../../redux/store';

export const useAppUpgradeEvent = () => {
  const appUpgradeEvent = useSelector((state) => state.appUpgrade.event);

  return appUpgradeEvent;
};
