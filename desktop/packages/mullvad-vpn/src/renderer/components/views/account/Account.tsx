import { useCallback, useEffect } from 'react';
import styled from 'styled-components';

import { urls } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Button, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useExclusiveTask } from '../../../lib/hooks/use-exclusive-task';
import { useEffectEvent } from '../../../lib/utility-hooks';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { RedeemVoucherButton } from '../../RedeemVoucher';
import { HeaderTitle } from '../../SettingsHeader';
import { AccountExpiryRow, AccountNumberRow, DeviceNameRow, LabelledRow } from './components';

const StyledViewContainer = styled(View.Container)`
  height: 100%;
  justify-content: space-between;
`;

export function Account() {
  const history = useHistory();
  const isOffline = useSelector((state) => state.connection.isBlocked);
  const { updateAccountData, openUrlWithAuth, logout } = useAppContext();

  const [buyMore] = useExclusiveTask(async () => {
    await openUrlWithAuth(urls.purchase);
  });

  const onMount = useEffectEvent(() => updateAccountData());
  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => onMount(), []);

  // Hack needed because if we just call `logout` directly in `onClick`
  // then it is run with the wrong `this`.
  const doLogout = useCallback(async () => {
    await logout();
  }, [logout]);

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={history.pop}>
        <AppNavigationHeader
          title={
            // TRANSLATORS: Title label in navigation bar
            messages.pgettext('account-view', 'Account')
          }
        />

        <View.Content>
          <StyledViewContainer flexDirection="column" horizontalMargin="medium">
            <FlexColumn gap="medium">
              <Text variant="titleBig">
                <HeaderTitle>{messages.pgettext('account-view', 'Account')}</HeaderTitle>
              </Text>

              <FlexColumn gap="large">
                <LabelledRow label={messages.pgettext('device-management', 'Device name')}>
                  <DeviceNameRow />
                </LabelledRow>

                <LabelledRow label={messages.pgettext('account-view', 'Account number')}>
                  <AccountNumberRow />
                </LabelledRow>

                <LabelledRow gap="tiny" label={messages.pgettext('account-view', 'Paid until')}>
                  <AccountExpiryRow />
                </LabelledRow>
              </FlexColumn>
            </FlexColumn>

            <FlexColumn gap="medium">
              <Button
                variant="success"
                disabled={isOffline}
                onClick={buyMore}
                aria-description={messages.pgettext('accessibility', 'Opens externally')}>
                <Button.Text>{messages.gettext('Buy more credit')}</Button.Text>
                <Button.Icon icon="external" />
              </Button>

              <RedeemVoucherButton />

              <Button variant="destructive" onClick={doLogout}>
                <Button.Text>
                  {
                    // TRANSLATORS: Button label for logging out.
                    messages.pgettext('account-view', 'Log out')
                  }
                </Button.Text>
              </Button>
            </FlexColumn>
          </StyledViewContainer>
        </View.Content>
      </BackAction>
    </View>
  );
}
