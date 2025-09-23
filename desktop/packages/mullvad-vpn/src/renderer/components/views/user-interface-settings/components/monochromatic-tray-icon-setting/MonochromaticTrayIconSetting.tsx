import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function MonochromaticTrayIconSetting() {
  const monochromaticIcon = useSelector((state) => state.settings.guiSettings.monochromaticIcon);
  const { setMonochromaticIcon } = useAppContext();

  return (
    <SettingsToggleListItem
      checked={monochromaticIcon}
      onCheckedChange={setMonochromaticIcon}
      description={messages.pgettext(
        'user-interface-settings-view',
        'Use a monochromatic tray icon instead of a colored one.',
      )}>
      <SettingsToggleListItem.Label>
        {messages.pgettext('user-interface-settings-view', 'Monochromatic tray icon')}
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}
