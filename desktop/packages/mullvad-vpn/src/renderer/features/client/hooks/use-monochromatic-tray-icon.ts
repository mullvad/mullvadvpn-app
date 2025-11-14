import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useMonochromaticTrayIcon() {
  const monochromaticIcon = useSelector((state) => state.settings.guiSettings.monochromaticIcon);
  const { setMonochromaticIcon } = useAppContext();
  return { monochromaticIcon, setMonochromaticIcon };
}
