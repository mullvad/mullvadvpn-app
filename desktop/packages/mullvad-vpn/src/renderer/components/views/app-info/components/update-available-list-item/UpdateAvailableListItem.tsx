import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Flex, Icon } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { ListItem } from '../../../../../lib/components/list-item';
import { useConnectionIsBlocked, usePushAppUpgrade, useVersionSuggestedUpgrade } from '../../hooks';

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

export function UpdateAvailableListItem() {
  const suggestedUpgrade = useVersionSuggestedUpgrade();
  const isBlocked = useConnectionIsBlocked();

  const pushAppUpgrade = usePushAppUpgrade();

  return (
    <ListItem disabled={isBlocked}>
      <ListItem.Item>
        <ListItem.Trigger onClick={pushAppUpgrade}>
          <ListItem.Content>
            <Flex $flexDirection="column">
              <ListItem.Label>
                {messages.pgettext('app-info-view', 'Update available')}
              </ListItem.Label>
              <StyledText variant="footnoteMini">{suggestedUpgrade}</StyledText>
            </Flex>
            <ListItem.Group>
              <Dot variant="warning" size="small" />
              <Icon icon="chevron-right" />
            </ListItem.Group>
          </ListItem.Content>
        </ListItem.Trigger>
      </ListItem.Item>
    </ListItem>
  );
}
