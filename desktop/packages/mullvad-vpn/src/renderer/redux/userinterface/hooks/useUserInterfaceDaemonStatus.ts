import { useSelector } from '../../store';

export const useUserInterfaceDaemonStatus = () => {
  return {
    daemonStatus: useSelector((state) => state.userInterface.daemonStatus),
  };
};
