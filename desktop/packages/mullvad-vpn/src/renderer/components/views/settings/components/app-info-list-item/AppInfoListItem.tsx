import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Flex, Icon } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { ListItem } from '../../../../../lib/components/list-item';
import { RoutePath } from '../../../../../lib/routes';
import { useVersionCurrent, useVersionSuggestedUpgrade } from '../../../../../redux/hooks';
import { NavigationListItem } from '../../../../NavigationListItem';

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

export function AppInfoListItem() {
  const { current } = useVersionCurrent();
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  return (
    <NavigationListItem to={RoutePath.appInfo}>
      <Flex $flexDirection="column">
        <ListItem.Label>
          {
            // TRANSLATORS: Navigation button to the 'App info' view
            messages.pgettext('settings-view', 'App info')
          }
        </ListItem.Label>
        {suggestedUpgrade && (
          <StyledText variant="footnoteMini">
            {
              // TRANSLATORS: Label for the app info list item indicating that an update is available and can be downloaded
              messages.pgettext('settings-view', 'Update available')
            }
          </StyledText>
        )}
      </Flex>
      <ListItem.Group>
        <ListItem.Text>{current}</ListItem.Text>
        {suggestedUpgrade && <Dot variant="warning" size="small" />}
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}
