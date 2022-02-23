import * as React from 'react';
import { sprintf } from 'sprintf-js';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { formatRelativeDate } from '../../shared/date-helper';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { IWgKey, WgKeyState } from '../redux/settings/reducers';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import ClipboardLabel from './ClipboardLabel';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { Layout } from './Layout';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import {
  StyledButtonRow,
  StyledContainer,
  StyledContent,
  StyledLastButtonRow,
  StyledMessage,
  StyledMessages,
  StyledNavigationScrollbars,
  StyledRow,
  StyledRowLabel,
  StyledRowLabelSpacer,
  StyledRowValue,
} from './WireguardKeysStyles';

export interface IProps {
  keyState: WgKeyState;
  isOffline: boolean;
  tunnelState: TunnelState;
  windowFocused: boolean;

  onClose: () => void;
  onGenerateKey: () => void;
  onReplaceKey: (old: IWgKey) => void;
  onVerifyKey: (publicKey: IWgKey) => void;
  onVisitWebsiteKey: () => Promise<void>;
}

export interface IState {
  recentlyGeneratedKey: boolean;
  userHasInitiatedVerification: boolean;
  ageOfKeyString: string;
}

export default class WireguardKeys extends React.Component<IProps, IState> {
  public state = {
    recentlyGeneratedKey: false,
    userHasInitiatedVerification: false,
    ageOfKeyString: WireguardKeys.ageOfKeyString(this.props.keyState),
  };

  private keyAgeUpdateInterval?: number;

  public static getDerivedStateFromProps(props: IProps) {
    return {
      ageOfKeyString: WireguardKeys.ageOfKeyString(props.keyState),
    };
  }

  public componentDidMount() {
    this.verifyKey();
    this.keyAgeUpdateInterval = window.setInterval(this.setAgeOfKeyStringState, 60 * 1000);
  }

  public componentWillUnmount() {
    clearInterval(this.keyAgeUpdateInterval);
  }

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
      <BackAction action={this.props.onClose}>
        <Layout>
          <StyledContainer>
            <NavigationContainer>
              <NavigationBar>
                <NavigationItems>
                  <TitleBarItem>
                    {
                      // TRANSLATORS: Title label in navigation bar
                      messages.pgettext('wireguard-keys-nav', 'WireGuard key')
                    }
                  </TitleBarItem>
                </NavigationItems>
              </NavigationBar>

              <StyledNavigationScrollbars fillContainer>
                <StyledContent>
                  <SettingsHeader>
                    <HeaderTitle>
                      {messages.pgettext('wireguard-keys-nav', 'WireGuard key')}
                    </HeaderTitle>
                  </SettingsHeader>

                  <StyledRow>
                    <StyledRowLabel>
                      <span>{messages.pgettext('wireguard-key-view', 'Public key')}</span>
                      <StyledRowLabelSpacer />
                      <span>{this.keyValidityLabel()}</span>
                    </StyledRowLabel>

                    <StyledRowValue>{this.getKeyText()}</StyledRowValue>
                  </StyledRow>
                  <StyledRow>
                    <StyledRowLabel>
                      {messages.pgettext('wireguard-key-view', 'Key generated')}
                    </StyledRowLabel>
                    <StyledRowValue>{this.state.ageOfKeyString}</StyledRowValue>
                  </StyledRow>

                  <StyledMessages>{this.getStatusMessage()}</StyledMessages>

                  <StyledButtonRow>{this.getGenerateButton()}</StyledButtonRow>
                  <StyledButtonRow>
                    <AppButton.BlueButton
                      disabled={this.isVerifyButtonDisabled()}
                      onClick={this.handleVerifyKeyPress}>
                      <AppButton.Label>
                        {messages.pgettext('wireguard-key-view', 'Verify key')}
                      </AppButton.Label>
                    </AppButton.BlueButton>
                  </StyledButtonRow>
                  <StyledLastButtonRow>
                    <AppButton.BlockingButton
                      disabled={this.props.isOffline}
                      onClick={this.props.onVisitWebsiteKey}>
                      <AriaDescriptionGroup>
                        <AriaDescribed>
                          <AppButton.BlueButton>
                            <AppButton.Label>
                              {messages.pgettext('wireguard-key-view', 'Manage keys')}
                            </AppButton.Label>
                            <AriaDescription>
                              <AppButton.Icon
                                source="icon-extLink"
                                height={16}
                                width={16}
                                aria-label={messages.pgettext('accessibility', 'Opens externally')}
                              />
                            </AriaDescription>
                          </AppButton.BlueButton>
                        </AriaDescribed>
                      </AriaDescriptionGroup>
                    </AppButton.BlockingButton>
                  </StyledLastButtonRow>
                </StyledContent>
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </StyledContainer>
        </Layout>
      </BackAction>
    );
  }

  private isVerifyButtonDisabled(): boolean {
    return this.props.keyState.type !== 'key-set';
  }

  private handleVerifyKeyPress = () => {
    this.setState({ userHasInitiatedVerification: true });
    this.verifyKey();
  };

  private verifyKey() {
    switch (this.props.keyState.type) {
      case 'key-set': {
        const key = this.props.keyState.key;
        this.props.onVerifyKey(key);
        break;
      }
      default:
        log.error(`onVerifyKey called from invalid state -  ${this.props.keyState.type}`);
    }
  }

  /// Action button can either generate or verify a key
  private getGenerateButton() {
    let buttonText = messages.pgettext('wireguard-key-view', 'Generate key');
    const regenerateText = messages.pgettext('wireguard-key-view', 'Regenerate key');

    let disabled = false;
    let generateKey = this.props.onGenerateKey;
    switch (this.props.keyState.type) {
      case 'key-set': {
        buttonText = regenerateText;
        const key = this.props.keyState.key;
        generateKey = () => this.props.onReplaceKey(key);
        break;
      }
      case 'being-verified':
        disabled = true;
        buttonText = regenerateText;
        break;
      case 'being-replaced':
      case 'being-generated':
        disabled = true;
        buttonText = messages.pgettext('wireguard-key-view', 'Generating key');
    }

    return (
      <AppButton.GreenButton disabled={disabled} onClick={generateKey}>
        <AppButton.Label>{buttonText}</AppButton.Label>
      </AppButton.GreenButton>
    );
  }

  private getKeyText() {
    switch (this.props.keyState.type) {
      case 'being-verified':
      case 'key-set': {
        // mimicking the truncating of the key from website
        const publicKey = this.props.keyState.key.publicKey;
        return (
          <StyledRowValue title={this.props.keyState.key.publicKey}>
            <ClipboardLabel
              value={publicKey}
              displayValue={publicKey.substring(0, 20) + '...'}
              obscureValue={false}
            />
          </StyledRowValue>
        );
      }
      case 'being-replaced':
      case 'being-generated':
        return <ImageView source="icon-spinner" height={19} width={19} />;
      default:
        return (
          <StyledRowValue>{messages.pgettext('wireguard-key-view', 'No key set')}</StyledRowValue>
        );
    }
  }

  private keyValidityLabel() {
    const keyStateType = this.props.keyState.type;
    if (keyStateType === 'being-verified' && this.state.userHasInitiatedVerification) {
      return <ImageView source="icon-spinner" height={20} width={20} />;
    } else if (this.props.keyState.type === 'key-set') {
      const valid = this.props.keyState.key.valid;
      const show = this.state.userHasInitiatedVerification || valid === false;
      return show && valid !== undefined ? (
        <StyledMessage success={valid}>
          {valid
            ? messages.pgettext('wireguard-key-view', 'Key is valid')
            : messages.pgettext('wireguard-key-view', 'Key is invalid')}
        </StyledMessage>
      ) : null;
    } else {
      return null;
    }
  }

  private static ageOfKeyString(keyState: WgKeyState): string {
    switch (keyState.type) {
      case 'key-set':
      case 'being-verified': {
        const createdDate = Math.min(Date.parse(keyState.key.created), Date.now());
        return formatRelativeDate(new Date(), createdDate, true);
      }
      default:
        return '-';
    }
  }

  private setAgeOfKeyStringState = () => {
    this.setState({
      ageOfKeyString: WireguardKeys.ageOfKeyString(this.props.keyState),
    });
  };

  private getStatusMessage() {
    if (this.props.isOffline && this.state.recentlyGeneratedKey) {
      return (
        <StyledMessage success={this.state.recentlyGeneratedKey}>
          {messages.pgettext('wireguard-key-view', 'Reconnecting with new WireGuard key...')}
        </StyledMessage>
      );
    } else {
      let message = '';
      switch (this.props.keyState.type) {
        case 'key-set': {
          const key = this.props.keyState.key;
          if (key.replacementFailure) {
            switch (key.replacementFailure) {
              case 'too_many_keys':
                message = this.formatKeygenFailure('too-many-keys');
                break;
              case 'generation_failure':
                message = this.formatKeygenFailure('generation-failure');
                break;
            }
          } else if (key.verificationFailed) {
            message = messages.pgettext('wireguard-key-view', 'Key verification failed');
          }

          break;
        }
        case 'too-many-keys':
        case 'generation-failure':
          message = this.formatKeygenFailure(this.props.keyState.type);
          break;
      }

      return <StyledMessage success={false}>{message}</StyledMessage>;
    }
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
