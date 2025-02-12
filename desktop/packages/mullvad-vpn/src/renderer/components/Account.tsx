import { useCallback, useEffect } from 'react';

import { formatDate, hasExpired } from '../../shared/account-expiry';
import { urls } from '../../shared/constants';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { Flex, Icon } from '../lib/components';
import { Spacings } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { useEffectEvent } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import AccountNumberLabel from './AccountNumberLabel';
import {
  AccountContainer,
  AccountOutOfTime,
  AccountRow,
  AccountRowLabel,
  AccountRows,
  AccountRowValue,
  DeviceRowValue,
} from './AccountStyles';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import DeviceInfoButton from './DeviceInfoButton';
import { BackAction } from './KeyboardNavigation';
import { Footer, Layout, SettingsContainer } from './Layout';
import { RedeemVoucherButton } from './RedeemVoucher';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

export default function Account() {
  const history = useHistory();
  const isOffline = useSelector((state) => state.connection.isBlocked);
  const { updateAccountData, openUrlWithAuth, logout } = useAppContext();

  const onBuyMore = useCallback(async () => {
    await openUrlWithAuth(urls.purchase);
  }, [openUrlWithAuth]);

  const onMount = useEffectEvent(() => updateAccountData());
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
              <AppButton.ButtonGroup>
                <AppButton.BlockingButton disabled={isOffline} onClick={onBuyMore}>
                  <AriaDescriptionGroup>
                    <AriaDescribed>
                      <AppButton.GreenButton>
                        <AppButton.Label>{messages.gettext('Buy more credit')}</AppButton.Label>
                        <AriaDescription>
                          <Icon
                            icon="external"
                            aria-label={messages.pgettext('accessibility', 'Opens externally')}
                          />
                        </AriaDescription>
                      </AppButton.GreenButton>
                    </AriaDescribed>
                  </AriaDescriptionGroup>
                </AppButton.BlockingButton>

                <RedeemVoucherButton />

                <AppButton.RedButton onClick={doLogout}>
                  {messages.pgettext('account-view', 'Log out')}
                </AppButton.RedButton>
              </AppButton.ButtonGroup>
            </Footer>
          </AccountContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function DeviceNameRow() {
  const deviceName = useSelector((state) => state.account.deviceName);
  return (
    <Flex $gap={Spacings.spacing3} $alignItems="center">
      <DeviceRowValue>{deviceName}</DeviceRowValue>
      <DeviceInfoButton />
    </Flex>
  );
}

function AccountNumberRow() {
  const accountNumber = useSelector((state) => state.account.accountNumber);
  return <AccountRowValue as={AccountNumberLabel} accountNumber={accountNumber || ''} />;
}

function AccountExpiryRow() {
  const accountExpiry = useSelector((state) => state.account.expiry);
  const expiryLocale = useSelector((state) => state.userInterface.locale);
  return <FormattedAccountExpiry expiry={accountExpiry} locale={expiryLocale} />;
}

function FormattedAccountExpiry(props: { expiry?: string; locale: string }) {
  if (props.expiry) {
    if (hasExpired(props.expiry)) {
      return (
        <AccountOutOfTime>{messages.pgettext('account-view', 'OUT OF TIME')}</AccountOutOfTime>
      );
    } else {
      return <AccountRowValue>{formatDate(props.expiry, props.locale)}</AccountRowValue>;
    }
  } else {
    return (
      <AccountRowValue>
        {messages.pgettext('account-view', 'Currently unavailable')}
      </AccountRowValue>
    );
  }
}
