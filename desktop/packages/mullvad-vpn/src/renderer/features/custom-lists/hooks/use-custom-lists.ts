import { useSelector } from '../../../redux/store';

export function useCustomLists() {
  const customLists = useSelector((state) => state.settings.customLists);

  return {
    customLists,
  };
}
