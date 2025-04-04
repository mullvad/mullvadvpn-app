import { useConnectionIsBlocked } from '../../../../../../redux/hooks';

export const useDisabled = () => {
  const { isBlocked } = useConnectionIsBlocked();

  const disabled = isBlocked;

  return disabled;
};
