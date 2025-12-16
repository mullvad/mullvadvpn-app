import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Flex, Text } from '../../../../../lib/components';
import { Link } from '../../../../../lib/components/link';
import { TransitionType, useHistory } from '../../../../../lib/history';
import { useSelector } from '../../../../../redux/store';

const StyledText = styled(Text)`
  text-transform: capitalize;
`;

export function DeviceNameRow() {
  const history = useHistory();
  const deviceName = useSelector((state) => state.account.deviceName);

  const navigateToManageDevices = React.useCallback(() => {
    history.push(RoutePath.manageDevices, { transition: TransitionType.push });
  }, [history]);

  return (
    <Flex justifyContent="space-between">
      <StyledText variant="bodySmallSemibold">{deviceName}</StyledText>
      <Link as="button" onClick={navigateToManageDevices}>
        <Link.Text>
          {
            // TRANSLATORS: Link text in the account view to navigate to the manage devices view.
            messages.pgettext('account-view', 'Manage devices')
          }
        </Link.Text>
      </Link>
    </Flex>
  );
}
