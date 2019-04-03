import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { messages } from '../../shared/gettext';
import AccountExpiry from '../lib/account-expiry';
import styles from './AccountStyles';
import * as AppButton from './AppButton';
import ClipboardLabel from './ClipboardLabel';
import { Container, Layout } from './Layout';
import { BackBarItem, NavigationBar } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

import { AccountToken } from '../../shared/daemon-rpc-types';

interface IProps {
  accountToken?: AccountToken;
  accountExpiry?: string;
  expiryLocale: string;
  isOffline: boolean;
  onLogout: () => void;
  onClose: () => void;
  onBuyMore: () => void;
}

export default class Account extends Component<IProps> {
  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.account}>
            <NavigationBar>
              <BackBarItem action={this.props.onClose}>
                {// TRANSLATORS: Back button in navigation bar
                messages.pgettext('account-nav', 'Settings')}
              </BackBarItem>
            </NavigationBar>

            <View style={styles.account__container}>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('account-view', 'Account')}</HeaderTitle>
              </SettingsHeader>

              <View style={styles.account__content}>
                <View style={styles.account__main}>
                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>
                      {messages.pgettext('account-view', 'Account ID')}
                    </Text>
                    <ClipboardLabel
                      style={styles.account__row_value}
                      value={this.props.accountToken || ''}
                      message={messages.pgettext('account-view', 'COPIED TO CLIPBOARD!')}
                    />
                  </View>

                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>
                      {messages.pgettext('account-view', 'Paid until')}
                    </Text>
                    <FormattedAccountExpiry
                      expiry={this.props.accountExpiry}
                      locale={this.props.expiryLocale}
                    />
                  </View>

                  <View style={styles.account__footer}>
                    <AppButton.GreenButton
                      style={styles.account__buy_button}
                      disabled={this.props.isOffline}
                      onPress={this.props.onBuyMore}>
                      <AppButton.Label>
                        {messages.pgettext('account-view', 'Buy more credit')}
                      </AppButton.Label>
                      <AppButton.Icon source="icon-extLink" height={16} width={16} />
                    </AppButton.GreenButton>
                    <AppButton.RedButton onPress={this.props.onLogout}>
                      {messages.pgettext('account-view', 'Log out')}
                    </AppButton.RedButton>
                  </View>
                </View>
              </View>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }
}

function FormattedAccountExpiry(props: { expiry?: string; locale: string }) {
  if (props.expiry) {
    const expiry = new AccountExpiry(props.expiry, props.locale);

    if (expiry.hasExpired()) {
      return (
        <Text style={styles.account__out_of_time}>
          {messages.pgettext('account-view', 'OUT OF TIME')}
        </Text>
      );
    } else {
      return <Text style={styles.account__row_value}>{expiry.formattedDate()}</Text>;
    }
  } else {
    return (
      <Text style={styles.account__row_value}>
        {messages.pgettext('account-view', 'Currently unavailable')}
      </Text>
    );
  }
}
