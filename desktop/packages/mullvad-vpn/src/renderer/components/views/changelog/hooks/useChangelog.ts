import { useUserInterfaceChangelog } from '../../../../redux/hooks';

export const useChangelog = () => {
  const { changelog } = useUserInterfaceChangelog();

  return changelog;
};
