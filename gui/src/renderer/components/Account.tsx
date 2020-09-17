import * as React from 'react';
import { formatDate, hasExpired } from '../../shared/account-expiry';
import { messages } from '../../shared/gettext';
import {
  AccountContainer,
  AccountFooter,
  AccountOutOfTime,
  AccountRow,
  AccountRowLabel,
  AccountRowValue,
  StyledBuyCreditButton,
  StyledContainer,
  StyledRedeemVoucherButton,
} from './AccountStyles';
import AccountTokenLabel from './AccountTokenLabel';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import { Layout } from './Layout';
import { ModalContainer } from './Modal';
import { BackBarItem, NavigationBar, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

import { AccountToken } from '../../shared/daemon-rpc-types';

interface IProps {
  accountToken?: AccountToken;
  accountExpiry?: string;
  expiryLocale: string;
  isOffline: boolean;
  onLogout: () => void;
  onClose: () => void;
  onBuyMore: () => Promise<void>;
}

export default class Account extends React.Component<IProps> {
  public render() {
    return (
      <ModalContainer>
        <Layout>
          <StyledContainer>
            <NavigationBar>
              <NavigationItems>
                <BackBarItem action={this.props.onClose}>
                  {
                    // TRANSLATORS: Back button in navigation bar
                    messages.pgettext('navigation-bar', 'Settings')
                  }
                </BackBarItem>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('account-view', 'Account')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <AccountContainer>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('account-view', 'Account')}</HeaderTitle>
              </SettingsHeader>

              <AccountRow>
                <AccountRowLabel>
                  {messages.pgettext('account-view', 'Account number')}
                </AccountRowLabel>
                <AccountRowValue
                  as={AccountTokenLabel}
                  accountToken={this.props.accountToken || ''}
                />
              </AccountRow>

              <AccountRow>
                <AccountRowLabel>{messages.pgettext('account-view', 'Paid until')}</AccountRowLabel>
                <FormattedAccountExpiry
                  expiry={this.props.accountExpiry}
                  locale={this.props.expiryLocale}
                />
              </AccountRow>

              <AccountFooter>
                <AppButton.BlockingButton
                  disabled={this.props.isOffline}
                  onClick={this.props.onBuyMore}>
                  <AriaDescriptionGroup>
                    <AriaDescribed>
                      <StyledBuyCreditButton>
                        <AppButton.Label>{messages.gettext('Buy more credit')}</AppButton.Label>
                        <AriaDescription>
                          <AppButton.Icon
                            source="icon-extLink"
                            height={16}
                            width={16}
                            aria-label={messages.pgettext('accessibility', 'Opens externally')}
                          />
                        </AriaDescription>
                      </StyledBuyCreditButton>
                    </AriaDescribed>
                  </AriaDescriptionGroup>
                </AppButton.BlockingButton>

                <StyledRedeemVoucherButton />

                <AppButton.RedButton onClick={this.props.onLogout}>
                  {messages.pgettext('account-view', 'Log out')}
                </AppButton.RedButton>
              </AccountFooter>
            </AccountContainer>
          </StyledContainer>
        </Layout>
      </ModalContainer>
    );
  }
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
