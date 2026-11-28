import { useSelector } from '../../store';

export const useIdentity = () => {
  return {
    identity: useSelector((state) => state.identity.identity),
  };
};
