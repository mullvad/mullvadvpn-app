import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Flex } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { useVersionCurrent, useVersionSuggestedUpgrade } from '../../../../../redux/hooks';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

const StyledText = styled(SettingsNavigationListItem.Text)`
  margin-top: -4px;
`;

export function AppInfoListItem() {
  const { current } = useVersionCurrent();
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  return (
    <SettingsNavigationListItem to={RoutePath.appInfo}>
      <Flex flexDirection="column">
        <SettingsNavigationListItem.Label>
          {
            // TRANSLATORS: Navigation button to the 'App info' view
            messages.pgettext('settings-view', 'App info')
          }
        </SettingsNavigationListItem.Label>
        {suggestedUpgrade && (
          <StyledText variant="footnoteMini">
            {
              // TRANSLATORS: Label for the app info list item indicating that an update is available and can be downloaded
              messages.pgettext('settings-view', 'Update available')
            }
          </StyledText>
        )}
      </Flex>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>{current}</SettingsNavigationListItem.Text>
        {suggestedUpgrade && <Dot variant="warning" size="small" />}
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}
