import { useCallback, useEffect } from 'react';

import { urls } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Button } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { useHistory } from '../../../lib/history';
import { useExclusiveTask } from '../../../lib/hooks/use-exclusive-task';
import { useEffectEvent } from '../../../lib/utility-hooks';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { Footer, Layout, SettingsContainer } from '../../Layout';
import { RedeemVoucherButton } from '../../RedeemVoucher';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { AccountContainer, AccountRow, AccountRowLabel, AccountRows } from './AccountStyles';
import { AccountExpiryRow, AccountNumberRow, DeviceNameRow } from './components';

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
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('account-view', 'Account')
            }
          />

          <AccountContainer>
            <SettingsHeader>
              <HeaderTitle>{messages.pgettext('account-view', 'Account')}</HeaderTitle>
            </SettingsHeader>

            <AccountRows>
              <AccountRow>
                <AccountRowLabel>
                  {messages.pgettext('device-management', 'Device name')}
                </AccountRowLabel>
                <DeviceNameRow />
              </AccountRow>

              <AccountRow>
                <AccountRowLabel>
                  {messages.pgettext('account-view', 'Account number')}
                </AccountRowLabel>
                <AccountNumberRow />
              </AccountRow>

              <AccountRow>
                <AccountRowLabel>{messages.pgettext('account-view', 'Paid until')}</AccountRowLabel>
                <AccountExpiryRow />
              </AccountRow>
            </AccountRows>

            <Footer>
              <FlexColumn $gap="medium">
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
            </Footer>
          </AccountContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
