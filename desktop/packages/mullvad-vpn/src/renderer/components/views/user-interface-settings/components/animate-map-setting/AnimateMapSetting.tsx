import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function AnimateMapSetting() {
  const animateMap = useSelector((state) => state.settings.guiSettings.animateMap);
  const { setAnimateMap } = useAppContext();

  return (
    <SettingsToggleListItem
      checked={animateMap}
      onCheckedChange={setAnimateMap}
      description={messages.pgettext('user-interface-settings-view', 'Animate map movements.')}>
      <SettingsToggleListItem.Label>
        {messages.pgettext('user-interface-settings-view', 'Animate map')}
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}
