import { useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useShowConnectionBlockedLabel = () => {
  const { isBlocked } = useConnectionIsBlocked();

  const showConnectionBlocked = isBlocked;

  return showConnectionBlocked;
};
