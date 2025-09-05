import { useLinuxApplicationRowContext } from '../LinuxApplicationRowContext';

export const useApplication = () => {
  const { application } = useLinuxApplicationRowContext();

  return application;
};
