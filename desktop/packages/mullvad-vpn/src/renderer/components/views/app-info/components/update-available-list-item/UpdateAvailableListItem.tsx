import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { useIsPlatformLinux } from '../../../../../hooks';
import { Flex, Icon } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { ListItem } from '../../../../../lib/components/list-item';
import { useConnectionIsBlocked, useVersionSuggestedUpgrade } from '../../../../../redux/hooks';
import { useHandleClick } from './hooks';

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

export function UpdateAvailableListItem() {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();
  const { isBlocked } = useConnectionIsBlocked();

  const isLinux = useIsPlatformLinux();
  const handleClick = useHandleClick();

  return (
    <ListItem disabled={isBlocked}>
      <ListItem.Item>
        <ListItem.Trigger onClick={handleClick}>
          <ListItem.Content>
            <Flex $flexDirection="column">
              <ListItem.Label>
                {
                  // TRANSLATORS: Label for update available list item.
                  messages.pgettext('app-info-view', 'Update available')
                }
              </ListItem.Label>
              <StyledText variant="footnoteMini">{suggestedUpgrade?.version}</StyledText>
            </Flex>
            <ListItem.Group>
              <Dot variant="warning" size="small" />
              <Icon icon={isLinux ? 'external' : 'chevron-right'} />
            </ListItem.Group>
          </ListItem.Content>
        </ListItem.Trigger>
      </ListItem.Item>
    </ListItem>
  );
}
