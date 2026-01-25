import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { useIsOn } from './hooks';

export type DaitaListItemProps = Omit<ListItemProps, 'children'>;

export function DaitaListItem(props: DaitaListItemProps) {
  const isOn = useIsOn();

  return (
    <SettingsNavigationListItem to={RoutePath.daitaSettings} {...props}>
      <SettingsNavigationListItem.Label>{strings.daita}</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Text>
          {isOn ? messages.gettext('On') : messages.gettext('Off')}
        </SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}
