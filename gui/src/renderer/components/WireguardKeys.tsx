import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { messages } from '../../shared/gettext';
import { WgKeyState } from '../redux/settings/reducers';
import * as AppButton from './AppButton';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import { BackBarItem, NavigationBar, NavigationContainer } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import styles from './WireguardKeysStyles';

export interface IProps {
  keyState: WgKeyState;
  isOffline: boolean;

  onGenerateKey: () => void;
  onVerifyKey: (publicKey: string) => void;
  onVisitWebsiteKey: () => void;
  onClose: () => void;
}

export default class WireguardKeys extends Component<IProps> {
  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.wgkeys}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose}>
                  {// TRANSLATORS: Back button in navigation bar
                  messages.pgettext('wireguard-keys-nav', 'Advanced')}
                </BackBarItem>
              </NavigationBar>
            </NavigationContainer>

            <View style={styles.wgkeys__container}>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-keys-nav', 'WireGuard key')}
                </HeaderTitle>
              </SettingsHeader>

              <View style={styles.wgkeys__row}>{this.blockedStateLabel()}</View>
              <View style={styles.wgkeys__row}>
                <Text style={styles.wgkeys__row_label}>
                  {messages.pgettext('wireguard-keys', 'Public key')}
                </Text>
                <View style={styles.wgkeys__row_value}>{this.getKeyRow()}</View>
                <View style={styles.wgkeys__validity_row}>{this.keyValidityLabel()}</View>
              </View>

              <View style={styles.wgkeys__row}>{this.getActionButton()}</View>
              <View style={styles.wgkeys__row}>
                <AppButton.GreenButton
                  disabled={this.props.isOffline}
                  onPress={this.props.onVisitWebsiteKey}>
                  <AppButton.Label>
                    {messages.pgettext('wireguard-key-view', 'Manage keys')}
                  </AppButton.Label>
                  <AppButton.Icon source="icon-extLink" height={16} width={16} />
                </AppButton.GreenButton>
              </View>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }

  /// Action button can either generate or verify a key
  private getActionButton() {
    switch (this.props.keyState.type) {
      case 'key-set':
        const publicKey = this.props.keyState.publicKey;
        // if the key is known to be invalid, allow the user to generate a new one
        if (this.props.keyState.valid === false) {
          break;
        }

        const verificationCallback = () => this.props.onVerifyKey(publicKey);

        return (
          <AppButton.GreenButton disabled={this.props.isOffline} onPress={verificationCallback}>
            <AppButton.Label>
              {messages.pgettext('wireguard-key-view', 'Verify key')}
            </AppButton.Label>
          </AppButton.GreenButton>
        );

      case 'being-verified':
        return this.busyButton(messages.pgettext('wireguard-key-view', 'Verifying key'));
      case 'being-generated':
        return this.busyButton(messages.pgettext('wireguard-key-view', 'Generating key'));
    }
    return (
      <AppButton.GreenButton disabled={this.props.isOffline} onPress={this.props.onGenerateKey}>
        <AppButton.Label>{messages.pgettext('wireguard-key-view', 'Generate key')}</AppButton.Label>
      </AppButton.GreenButton>
    );
  }

  private busyButton(message: string) {
    return (
      <AppButton.GreenButton disabled={true}>
        <AppButton.Label>{message}</AppButton.Label>
        <AppButton.Icon source="icon-spinner" height={16} width={16} />
      </AppButton.GreenButton>
    );
  }

  private getKeyRow() {
    switch (this.props.keyState.type) {
      case 'being-verified':
      case 'key-set':
        // mimicking the truncating of the key from website
        return (
          <View title={this.props.keyState.publicKey}>
            <Text style={styles.wgkeys__row_value}>
              {this.props.keyState.publicKey.substring(0, 20) + '...'}
            </Text>
          </View>
        );
      case 'being-generated':
        return <ImageView source="icon-spinner" height={25} width={25} />;
      case 'too-many-keys':
        return (
          <Text style={styles.wgkeys__invalid_key}>
            {messages.pgettext('wireguard-key-view', 'Account has too many keys already')}
          </Text>
        );
      case 'generation-failure':
        return (
          <Text style={styles.wgkeys__invalid_key}>
            {messages.pgettext('wireguard-key-view', 'Failed to generate key')}
          </Text>
        );
      default:
        return (
          <Text style={styles.wgkeys__row_value}>
            {messages.pgettext('wireguard-key-view', 'No key set')}
          </Text>
        );
    }
  }

  private keyValidityLabel() {
    switch (this.props.keyState.type) {
      case 'being-verified':
        return <ImageView source="icon-spinner" height={25} width={25} />;
      case 'key-set':
        if (this.props.keyState.valid === true) {
          return (
            <Text style={styles.wgkeys__valid_key}>
              {messages.pgettext('account-view', 'Key is valid')}
            </Text>
          );
        } else if (this.props.keyState.valid === false) {
          return (
            <Text style={styles.wgkeys__invalid_key}>
              {messages.pgettext('wireguard-key-view', 'Key is invalid')}
            </Text>
          );
        }
      default:
        return '';
    }
  }

  private blockedStateLabel() {
    if (!this.props.isOffline) {
      return undefined;
    }
    return (
      <Text style={styles.wgkeys__invalid_key}>
        {messages.pgettext('wireguard-key-view', "Can't manage keys whilst in a blocked state")}
      </Text>
    );
  }
}
