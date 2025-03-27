import { useUserInterfaceIsMacOs13OrNewer } from '../../../../redux/hooks';

export const useShowSplitTunneling = () => {
  const { isMacOs13OrNewer } = useUserInterfaceIsMacOs13OrNewer();
  return window.env.platform !== 'darwin' || isMacOs13OrNewer;
};
