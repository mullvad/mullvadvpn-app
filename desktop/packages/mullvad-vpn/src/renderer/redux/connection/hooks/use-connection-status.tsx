import { useSelector } from '../../store';

export const useConnectionStatus = () => {
  return { status: useSelector((state) => state.connection.status) };
};
