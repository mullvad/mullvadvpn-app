import { useSelector } from '../../../redux/store';

export const useConnection = () => {
  const connection = useSelector((state) => state.connection);
  return connection;
};
