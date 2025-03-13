import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../shared/gettext';
import { Flex, Icon } from '../../../../lib/components';
import { ListItem } from '../../../../lib/components/list-item';
import { Notification } from '../../../../lib/components/notification';
import { useHistory } from '../../../../lib/history';
import { RoutePath } from '../../../../lib/routes';
import { useSelector } from '../../../../redux/store';

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

export const UpdateAvailableListItem = () => {
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);
  const isOffline = useSelector((state) => state.connection.isBlocked);
  const history = useHistory();

  return (
    <>
      <ListItem disabled={isOffline}>
        <ListItem.Item>
          <ListItem.Trigger onClick={navigate}>
            <ListItem.Content>
              <Flex $flexDirection="column">
                <ListItem.Label>
                  {messages.pgettext('app-info-view', 'Update available')}
                </ListItem.Label>
                <StyledText variant="footnoteMini">{suggestedUpgrade?.version}</StyledText>
              </Flex>
              <ListItem.Group>
                <Notification variant="warning" size="small" />
                <Icon icon="chevron-right" />
              </ListItem.Group>
            </ListItem.Content>
          </ListItem.Trigger>
        </ListItem.Item>
      </ListItem>
    </>
  );
};
