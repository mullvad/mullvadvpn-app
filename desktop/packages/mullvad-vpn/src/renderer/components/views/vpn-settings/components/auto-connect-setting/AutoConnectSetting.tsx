import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useScrollToListItem } from '../../../../../hooks';
import { useSelector } from '../../../../../redux/store';
import { ToggleListItem } from '../../../../toggle-list-item';

export function AutoConnectSetting() {
  const autoConnect = useSelector((state) => state.settings.guiSettings.autoConnect);
  const { setAutoConnect } = useAppContext();
  const { animation } = useScrollToListItem();

  const footer = messages.pgettext(
    'vpn-settings-view',
    'Automatically connect to a server when the app launches.',
  );

  return (
    <ToggleListItem
      animation={animation}
      checked={autoConnect}
      onCheckedChange={setAutoConnect}
      footer={footer}>
      <ToggleListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Auto-connect')}
      </ToggleListItem.Label>
      <ToggleListItem.Switch aria-description={footer} />
    </ToggleListItem>
  );
}
