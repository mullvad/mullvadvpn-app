import { useSelector } from '../../store';

export const useUserInterfaceIsMacOs13OrNewer = () => {
  return {
    isMacOs13OrNewer: useSelector((state) => state.userInterface.isMacOs13OrNewer),
  };
};
