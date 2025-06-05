import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Flex } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { useVersionCurrent, useVersionSuggestedUpgrade } from '../../../../../redux/hooks';
import { NavigationListItem } from '../../../../NavigationListItem';

const StyledText = styled(NavigationListItem.Text)`
  margin-top: -4px;
`;

export function AppInfoListItem() {
  const { current } = useVersionCurrent();
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  return (
    <NavigationListItem to={RoutePath.appInfo}>
      <Flex $flexDirection="column">
        <NavigationListItem.Label>
          {
            // TRANSLATORS: Navigation button to the 'App info' view
            messages.pgettext('settings-view', 'App info')
          }
        </NavigationListItem.Label>
        {suggestedUpgrade && (
          <StyledText variant="footnoteMini">
            {
              // TRANSLATORS: Label for the app info list item indicating that an update is available and can be downloaded
              messages.pgettext('settings-view', 'Update available')
            }
          </StyledText>
        )}
      </Flex>
      <NavigationListItem.Group>
        <NavigationListItem.Text>{current}</NavigationListItem.Text>
        {suggestedUpgrade && <Dot variant="warning" size="small" />}
        <NavigationListItem.Icon icon="chevron-right" />
      </NavigationListItem.Group>
    </NavigationListItem>
  );
}
