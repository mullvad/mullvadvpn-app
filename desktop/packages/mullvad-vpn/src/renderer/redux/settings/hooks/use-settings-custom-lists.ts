import { useSelector } from '../../../redux/store';

export function useSettingsCustomLists() {
  const customLists = useSelector((state) => state.settings.customLists);

  return {
    customLists,
  };
}
