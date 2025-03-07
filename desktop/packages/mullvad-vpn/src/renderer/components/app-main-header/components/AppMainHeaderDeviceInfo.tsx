import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { closeToExpiry, formatRemainingTime, hasExpired } from '../../../../shared/account-expiry';
import { messages } from '../../../../shared/gettext';
import { capitalizeEveryWord } from '../../../../shared/string-helpers';
import { Flex, FootnoteMini } from '../../../lib/components';
import { Colors } from '../../../lib/foundations';
import { useSelector } from '../../../redux/store';

const StyledTimeLeftLabel = styled(FootnoteMini)({
  whiteSpace: 'nowrap',
});

const StyledDeviceLabel = styled(FootnoteMini)({
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
});

export const AppMainHeaderDeviceInfo = () => {
  const deviceName = useSelector((state) => state.account.deviceName);
  const accountExpiry = useSelector((state) => state.account.expiry);
  const isOutOfTime = accountExpiry ? hasExpired(accountExpiry) : false;
  const formattedExpiry = isOutOfTime
    ? sprintf(messages.ngettext('1 day', '%d days', 0), 0)
    : accountExpiry
      ? formatRemainingTime(accountExpiry)
      : '';

  return (
    <Flex $gap="large" $margin={{ top: 'tiny' }}>
      <StyledDeviceLabel color={Colors.white80}>
        {sprintf(
          // TRANSLATORS: A label that will display the newly created device name to inform the user
          // TRANSLATORS: about it.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(deviceName)s - The name of the current device
          messages.pgettext('device-management', 'Device name: %(deviceName)s'),
          {
            deviceName: capitalizeEveryWord(deviceName ?? ''),
          },
        )}
      </StyledDeviceLabel>
      {accountExpiry && !closeToExpiry(accountExpiry) && !isOutOfTime && (
        <StyledTimeLeftLabel color={Colors.white80}>
          {sprintf(messages.pgettext('device-management', 'Time left: %(timeLeft)s'), {
            timeLeft: formattedExpiry,
          })}
        </StyledTimeLeftLabel>
      )}
    </Flex>
  );
};
