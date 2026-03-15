import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { useSelector } from '../../../../../redux/store';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { getObfuscationLabel } from './utils';

export type AntiCensorshipListItemProps = Omit<ListItemProps, 'children'>;

export function AntiCensorshipListItem(props: AntiCensorshipListItemProps) {
  const { selectedObfuscation } = useSelector((state) => state.settings.obfuscationSettings);
  const obfuscationLabel = getObfuscationLabel(selectedObfuscation);

  return (
    <SettingsNavigationListItem to={RoutePath.antiCensorship} {...props}>
      <SettingsNavigationListItem.Item>
        <SettingsNavigationListItem.Item.Label>
          {
            // TRANSLATORS: Label for list item that navigates to anti-censorship
            // TRANSLATORS: settings view.
            messages.pgettext('vpn-settings-view', 'Anti-censorship')
          }
        </SettingsNavigationListItem.Item.Label>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Text>
            {obfuscationLabel}
          </SettingsNavigationListItem.Item.Text>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}
