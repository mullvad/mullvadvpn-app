import { useSelector } from '../../store';
import { setAppUpgradeEvent } from '../actions';

export const useAppUpgradeEvent = () => {
  return {
    appUpgradeEvent: useSelector((state) => state.appUpgrade.event),
    setAppUpgradeEvent,
  };
};
