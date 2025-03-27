import { useConnectionStatus } from '../../../../../../redux/hooks';

export const useIsConnected = () => {
  const { status } = useConnectionStatus();
  return status.state === 'connected';
};
