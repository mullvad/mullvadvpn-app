import { useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { closeToExpiry, formatRemainingTime, hasExpired } from '../../../../shared/account-expiry';
import { messages } from '../../../../shared/gettext';
import { Flex, FootnoteMini } from '../../../lib/components';
import { useInterval } from '../../../lib/hooks';
import { formatDeviceName } from '../../../lib/utils';
import { useSelector } from '../../../redux/store';

const StyledTimeLeftLabel = styled(FootnoteMini)({
  whiteSpace: 'nowrap',
});

const StyledDeviceLabel = styled(FootnoteMini)({
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
});

const StyledFlex = styled(Flex)({
  width: '100%',
});

export const AppMainHeaderDeviceInfo = () => {
  const deviceName = useSelector((state) => state.account.deviceName);
  const accountExpiry = useSelector((state) => state.account.expiry);
  const isOutOfTime = accountExpiry ? hasExpired(accountExpiry) : false;

  const [timeLeft, setTimeLeft] = useState(formatTimeLeft(accountExpiry));

  // The time left value must be recalculated recurringly since it should change when time passes.
  useInterval(() => setTimeLeft(formatTimeLeft(accountExpiry)), 60 * 60 * 1_000);

  // The time left value must be updated every time the accountExpiry changes.
  useEffect(() => {
    setTimeLeft(formatTimeLeft(accountExpiry));
  }, [accountExpiry]);

  return (
    <StyledFlex $gap="large" $margin={{ top: 'tiny' }}>
      <StyledDeviceLabel color="whiteAlpha80">
        {sprintf(
          // TRANSLATORS: A label that will display the newly created device name to inform the user
          // TRANSLATORS: about it.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(deviceName)s - The name of the current device
          messages.pgettext('device-management', 'Device name: %(deviceName)s'),
          {
            deviceName: formatDeviceName(deviceName ?? ''),
          },
        )}
      </StyledDeviceLabel>
      {accountExpiry && !closeToExpiry(accountExpiry) && !isOutOfTime && (
        <StyledTimeLeftLabel color="whiteAlpha80">
          {sprintf(messages.pgettext('device-management', 'Time left: %(timeLeft)s'), {
            timeLeft,
          })}
        </StyledTimeLeftLabel>
      )}
    </StyledFlex>
  );
};

function formatTimeLeft(accountExpiry?: string): string {
  const isOutOfTime = accountExpiry ? hasExpired(accountExpiry) : false;
  return isOutOfTime
    ? sprintf(messages.ngettext('1 day', '%d days', 0), 0)
    : accountExpiry
      ? formatRemainingTime(accountExpiry)
      : '';
}
