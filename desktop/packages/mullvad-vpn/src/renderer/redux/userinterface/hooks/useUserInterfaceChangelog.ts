import { useSelector } from '../../store';

export const useUserInterfaceChangelog = () => {
  return {
    changelog: useSelector((state) => state.userInterface.changelog),
  };
};
