import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Flex, Icon } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { ListItem } from '../../../../../lib/components/list-item';
import { useHistory } from '../../../../../lib/history';
import { RoutePath } from '../../../../../lib/routes';
import { useSelector } from '../../../../../redux/store';

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

export function UpdateAvailableListItem() {
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);
  const isOffline = useSelector((state) => state.connection.isBlocked);
  const history = useHistory();
  // TODO: Change to in app upgrade view
  const navigate = useCallback(() => history.push(RoutePath.account), [history]);

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
    </>
  );
}
