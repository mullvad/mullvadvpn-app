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
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Label for list item that navigates to anti-censorship
          // TRANSLATORS: settings view.
          messages.pgettext('vpn-settings-view', 'Anti-censorship')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Text>{obfuscationLabel}</SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}
