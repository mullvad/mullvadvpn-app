import { useApplicationRowContext } from '../ApplicationRowContext';

export const useApplication = () => {
  const { application } = useApplicationRowContext();

  return application;
};
