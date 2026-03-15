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
      <SettingsNavigationListItem.Item>
        <FlexColumn>
          <SettingsNavigationListItem.Item.Label>
            {
              // TRANSLATORS: Navigation button to the 'App info' view
              messages.pgettext('settings-view', 'App info')
            }
          </SettingsNavigationListItem.Item.Label>
          {suggestedUpgrade && (
            <SettingsNavigationListItem.Item.Text variant="footnoteMini">
              {
                // TRANSLATORS: Label for the app info list item indicating that an update is available and can be downloaded
                messages.pgettext('settings-view', 'Update available')
              }
            </SettingsNavigationListItem.Item.Text>
          )}
        </FlexColumn>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Group gap="small">
            <SettingsNavigationListItem.Item.Text>{current}</SettingsNavigationListItem.Item.Text>
            {suggestedUpgrade && <Dot variant="warning" size="small" />}
          </SettingsNavigationListItem.Item.Group>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}
