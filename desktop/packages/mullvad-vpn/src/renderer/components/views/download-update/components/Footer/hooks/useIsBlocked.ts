import { useSelector } from '../../../../../../redux/store';

export const useIsBlocked = () => {
  const isBlocked = useSelector((state) => state.connection.isBlocked);

  return isBlocked;
};
