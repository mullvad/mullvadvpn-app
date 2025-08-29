import { useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useShowConnectionBlocked = () => {
  const { isBlocked } = useConnectionIsBlocked();

  const showConnectionBlocked = isBlocked;

  return showConnectionBlocked;
};
