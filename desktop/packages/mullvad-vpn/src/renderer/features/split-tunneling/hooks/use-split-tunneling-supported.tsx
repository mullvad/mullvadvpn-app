import { useSelector } from '../../../redux/store';

export const useSplitTunnelingSupported = () => {
  return {
    splitTunnelingSupported: useSelector((state) => state.settings.splitTunnelingSupported),
  };
};
