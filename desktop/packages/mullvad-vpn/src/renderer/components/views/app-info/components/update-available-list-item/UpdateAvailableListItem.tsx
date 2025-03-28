import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { usePushAppUpgrade } from '../../../../../history/hooks';
import { Flex, Icon } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { ListItem, ListItemProps } from '../../../../../lib/components/list-item';
import { useConnectionIsBlocked, useVersionSuggestedUpgrade } from '../../../../../redux/hooks';
import { useIsLinux, useOpenDownloadUrl } from './hooks';

const StyledText = styled(ListItem.Text)`
  margin-top: -4px;
`;

type UpdateAvailableListItemProps = Omit<ListItemProps, 'children'>;

export function UpdateAvailableListItem(props: UpdateAvailableListItemProps) {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();
  const { isBlocked } = useConnectionIsBlocked();

  const openDownloadUrl = useOpenDownloadUrl();
  const pushAppUpgrade = usePushAppUpgrade();
  const isLinux = useIsLinux();
  const onClick = isLinux ? openDownloadUrl : pushAppUpgrade;

  return (
    <ListItem disabled={isBlocked} {...props}>
      <ListItem.Item>
        <ListItem.Trigger onClick={onClick}>
          <ListItem.Content>
            <Flex $flexDirection="column">
              <ListItem.Label>
                {
                  // TRANSLATORS: Label for update available list item.
                  messages.pgettext('app-info-view', 'Update available')
                }
              </ListItem.Label>
              <StyledText variant="footnoteMini">{suggestedUpgrade}</StyledText>
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
