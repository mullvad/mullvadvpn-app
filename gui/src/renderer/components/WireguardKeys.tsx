import log from 'electron-log';
import moment from 'moment';
import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { sprintf } from 'sprintf-js';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { IWgKey, WgKeyState } from '../redux/settings/reducers';
import * as AppButton from './AppButton';
import ClipboardLabel from './ClipboardLabel';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import { BackBarItem, NavigationBar, NavigationContainer, NavigationItems } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import styles from './WireguardKeysStyles';

export interface IProps {
  keyState: WgKeyState;
  isOffline: boolean;
  locale: string;
  tunnelState: TunnelState;

  onClose: () => void;
  onGenerateKey: () => void;
  onReplaceKey: (old: IWgKey) => void;
  onVerifyKey: (publicKey: IWgKey) => void;
  onVisitWebsiteKey: () => Promise<void>;
}

export interface IState {
  recentlyGeneratedKey: boolean;
}

export default class WireguardKeys extends Component<IProps, IState> {
  public state = {
    recentlyGeneratedKey: false,
  };

  public componentDidUpdate(prevProps: IProps) {
    const prevKey =
      prevProps.keyState.type === 'key-set' ? prevProps.keyState.key.publicKey : undefined;
    const key =
      this.props.keyState.type === 'key-set' ? this.props.keyState.key.publicKey : undefined;
    if (this.props.tunnelState.state === 'connected' && key !== undefined && key != prevKey) {
      this.setState({ recentlyGeneratedKey: true });
    }

    if (
      this.state.recentlyGeneratedKey &&
      prevProps.tunnelState.state !== 'connected' &&
      this.props.tunnelState.state === 'connected'
    ) {
      this.setState({ recentlyGeneratedKey: false });
    }
  }

  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.wgkeys}>
            <NavigationContainer>
              <NavigationBar>
                <NavigationItems>
                  <BackBarItem action={this.props.onClose}>
                    {
                      // TRANSLATORS: Back button in navigation bar
                      messages.pgettext('wireguard-keys-nav', 'Advanced')
                    }
                  </BackBarItem>
                </NavigationItems>
              </NavigationBar>
            </NavigationContainer>

            <View style={styles.wgkeys__container}>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-keys-nav', 'WireGuard key')}
                </HeaderTitle>
              </SettingsHeader>

              <View style={styles.wgkeys__row}>
                <Text style={styles.wgkeys__row_label}>
                  {messages.pgettext('wireguard-keys', 'Public key')}
                </Text>

                <View style={styles.wgkeys__row_value}>{this.getKeyText()}</View>
              </View>
              <View style={styles.wgkeys__row}>
                <Text style={styles.wgkeys__row_label}>
                  {messages.pgettext('wireguard-keys', 'Key generated')}
                </Text>
                <Text style={styles.wgkeys__row_value}>{this.ageOfKeyString()}</Text>
              </View>

              <View style={styles.wgkeys__messages}>
                {this.props.isOffline ? (
                  this.offlineLabel()
                ) : (
                  <View style={styles.wgkeys__row}>{this.keyValidityLabel()}</View>
                )}
              </View>

              <View style={styles.wgkeys__row}>{this.getGenerateButton()}</View>
              <View style={styles.wgkeys__row}>
                <AppButton.BlueButton
                  disabled={this.isVerifyButtonDisabled()}
                  onPress={this.getOnVerifyKeyCb()}>
                  <AppButton.Label>
                    {messages.pgettext('wireguard-key-view', 'Verify key')}
                  </AppButton.Label>
                </AppButton.BlueButton>
              </View>
              <View style={styles.wgkeys__row}>
                <AppButton.BlockingButton
                  disabled={this.props.isOffline}
                  onPress={this.props.onVisitWebsiteKey}>
                  <AppButton.BlueButton>
                    <AppButton.Label>
                      {messages.pgettext('wireguard-key-view', 'Manage keys')}
                    </AppButton.Label>
                    <AppButton.Icon source="icon-extLink" height={16} width={16} />
                  </AppButton.BlueButton>
                </AppButton.BlockingButton>
              </View>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }

  private offlineLabel() {
    return this.state.recentlyGeneratedKey ? (
      <Text style={[styles.wgkeys__row, styles.wgkeys__valid_key]}>
        {messages.pgettext('wireguard-key-view', 'Reconnecting with new WireGuard key...')}
      </Text>
    ) : (
      <Text style={[styles.wgkeys__row, styles.wgkeys__invalid_key]}>
        {messages.pgettext('wireguard-key-view', 'Unable to manage keys while in a blocked state')}
      </Text>
    );
  }

  private isVerifyButtonDisabled(): boolean {
    switch (this.props.keyState.type) {
      case 'key-set':
        return false || this.props.isOffline;
      default:
        return true;
    }
  }

  private getOnVerifyKeyCb() {
    return () => {
      switch (this.props.keyState.type) {
        case 'key-set': {
          const key = this.props.keyState.key;
          this.props.onVerifyKey(key);
          break;
        }
        default:
          log.error(`onVerifyKey called from invalid state -  ${this.props.keyState.type}`);
      }
    };
  }

  /// Action button can either generate or verify a key
  private getGenerateButton() {
    const generateText = messages.pgettext('wireguard-key-view', 'Generate key');
    const regenerateText = messages.pgettext('wireguard-key-view', 'Regenerate key');
    let buttonText = generateText;

    let generateKey = this.props.onGenerateKey;
    switch (this.props.keyState.type) {
      case 'key-set': {
        buttonText = regenerateText;
        const key = this.props.keyState.key;
        generateKey = () => this.props.onReplaceKey(key);
        break;
      }
      case 'being-verified':
        return this.busyButton(regenerateText);
      case 'being-replaced':
      case 'being-generated':
        return this.busyButton(messages.pgettext('wireguard-key-view', 'Generating key'));
    }
    return (
      <AppButton.GreenButton disabled={this.props.isOffline} onPress={generateKey}>
        <AppButton.Label>{buttonText}</AppButton.Label>
      </AppButton.GreenButton>
    );
  }

  private busyButton(message: string) {
    return (
      <AppButton.BlueButton disabled={true}>
        <AppButton.Label>{message}</AppButton.Label>
        <AppButton.Icon source="icon-spinner" height={16} width={16} />
      </AppButton.BlueButton>
    );
  }

  private getKeyText() {
    switch (this.props.keyState.type) {
      case 'being-verified':
      case 'key-set': {
        // mimicking the truncating of the key from website
        const publicKey = this.props.keyState.key.publicKey;
        return (
          <View title={this.props.keyState.key.publicKey}>
            <Text style={styles.wgkeys__row_value}>
              <ClipboardLabel value={publicKey} displayValue={publicKey.substring(0, 20) + '...'} />
            </Text>
          </View>
        );
      }
      case 'being-replaced':
      case 'being-generated':
        return <ImageView source="icon-spinner" height={19} width={19} />;
      case 'too-many-keys':
      case 'generation-failure':
        return (
          <Text style={styles.wgkeys__invalid_key}>
            {this.formatKeygenFailure(this.props.keyState.type)}
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
        return <ImageView source="icon-spinner" height={20} width={20} />;
      case 'key-set': {
        const key = this.props.keyState.key;
        if (key.valid === true) {
          return (
            <Text style={styles.wgkeys__valid_key}>
              {messages.pgettext('account-view', 'Key is valid')}
            </Text>
          );
        } else if (key.valid === false) {
          return (
            <Text style={styles.wgkeys__invalid_key}>
              {messages.pgettext('wireguard-key-view', 'Key is invalid')}
            </Text>
          );
        } else if (key.replacementFailure) {
          let failureMessage = '';
          switch (key.replacementFailure) {
            case 'too_many_keys':
              failureMessage = this.formatKeygenFailure('too-many-keys');
              break;
            case 'generation_failure':
              failureMessage = this.formatKeygenFailure('generation-failure');
              break;
          }

          return <Text style={styles.wgkeys__invalid_key}>{failureMessage}</Text>;
        } else if (key.verificationFailed) {
          return (
            <Text style={styles.wgkeys__invalid_key}>
              {messages.pgettext('wireguard-key-view', 'Key verification failed')}
            </Text>
          );
        } else {
          return null;
        }
      }
      default:
        return null;
    }
  }

  private ageOfKeyString(): string {
    let keyCreatedSince = '-';
    switch (this.props.keyState.type) {
      case 'key-set':
      case 'being-verified':
        keyCreatedSince = moment(this.props.keyState.key.created)
          .locale(this.props.locale)
          .fromNow();
        break;
    }

    return keyCreatedSince;
  }

  private formatKeygenFailure(failure: 'too-many-keys' | 'generation-failure'): string {
    switch (failure) {
      case 'too-many-keys':
        // TRANSLATORS: "%(manage)" is replaced with the text in the "Manage keys" button.
        return sprintf(
          messages.pgettext(
            'wireguard-key-view',
            'Unable to regenerate key: you already have the maximum number of keys. To generate a new key, you first need to revoke one under “Manage keys.”',
          ),
          { manage: messages.pgettext('wireguard-key-view', 'Manage keys') },
        );
      case 'generation-failure':
        return messages.pgettext('wireguard-key-view', 'Failed to generate a key');
      default:
        return failure;
    }
  }
}
