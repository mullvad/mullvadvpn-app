import { useSelector } from '../../store';
import { setAppUpgradeEvent } from '../actions';

export const useAppUpgradeEvent = () => {
  return {
    event: useSelector((state) => state.appUpgrade.event),
    setAppUpgradeEvent,
  };
};
