import { useSelector } from '../../../redux/store';

export function useMonochromaticTrayIcon() {
  const monochromaticIcon = useSelector((state) => state.settings.guiSettings.monochromaticIcon);
  return monochromaticIcon;
}
