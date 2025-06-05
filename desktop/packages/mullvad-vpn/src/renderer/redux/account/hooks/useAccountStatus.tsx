import { useSelector } from '../../store';

export const useAccountStatus = () => {
  return {
    status: useSelector((state) => state.account.status),
  };
};
