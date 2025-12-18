import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Flex, Icon } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { ListItem } from '../../../../../lib/components/list-item';
import { useVersionSuggestedUpgrade } from '../../../../../redux/hooks';
import { isPlatform } from '../../../../../utils';
import { useHandleClick } from './hooks';

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

export function UpdateAvailableListItem() {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  const isLinux = isPlatform('linux');
  const handleClick = useHandleClick();

  return (
    <ListItem>
      <ListItem.Trigger onClick={handleClick}>
        <ListItem.Item>
          <ListItem.Content>
            <Flex flexDirection="column">
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
        </ListItem.Item>
      </ListItem.Trigger>
    </ListItem>
  );
}
