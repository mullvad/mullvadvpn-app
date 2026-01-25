import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Dot } from '../../../../../lib/components/dot';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { useVersionCurrent, useVersionSuggestedUpgrade } from '../../../../../redux/hooks';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type AppInfoListItemProps = Omit<ListItemProps, 'children'>;

export function AppInfoListItem(props: AppInfoListItemProps) {
  const { current } = useVersionCurrent();
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  return (
    <SettingsNavigationListItem to={RoutePath.appInfo} {...props}>
      <FlexColumn>
        <SettingsNavigationListItem.Label>
          {
            // TRANSLATORS: Navigation button to the 'App info' view
            messages.pgettext('settings-view', 'App info')
          }
        </SettingsNavigationListItem.Label>
        {suggestedUpgrade && (
          <SettingsNavigationListItem.Text variant="footnoteMini">
            {
              // TRANSLATORS: Label for the app info list item indicating that an update is available and can be downloaded
              messages.pgettext('settings-view', 'Update available')
            }
          </SettingsNavigationListItem.Text>
        )}
      </FlexColumn>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Group gap="small">
          {suggestedUpgrade && <Dot variant="warning" size="small" />}
          <SettingsNavigationListItem.Text>{current}</SettingsNavigationListItem.Text>
        </SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}
