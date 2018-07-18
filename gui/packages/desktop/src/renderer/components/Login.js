// @flow

import * as React from 'react';
import { Component, Text, View, Animated, Styles, UserInterface } from 'reactxp';
import { Layout, Container, Header } from './Layout';
import { SettingsBarButton, Brand } from './HeaderBar';
import { AccountInput, Accordion } from '@mullvad/components';
import Img from './Img';
import * as Cell from './Cell';
import * as AppButton from './AppButton';
import styles from './LoginStyles';
import { colors } from '../../config';

import type { LoginState } from '../redux/account/reducers';
import type { AccountToken } from '../lib/daemon-rpc';

export type Props = {
  accountToken: ?AccountToken,
  accountHistory: Array<AccountToken>,
  loginError: ?Error,
  loginState: LoginState,
  openSettings: ?() => void,
  openExternalLink: (type: string) => void,
  login: (accountToken: AccountToken) => void,
  resetLoginError: () => void,
  updateAccountToken: (accountToken: AccountToken) => void,
  removeAccountTokenFromHistory: (accountToken: AccountToken) => Promise<void>,
};

type State = {
  isActive: boolean,
};

const MIN_ACCOUNT_TOKEN_LENGTH = 10;

export default class Login extends Component<Props, State> {
  state = {
    isActive: true,
  };

  _accountInput: ?AccountInput;
  _shouldResetLoginError = false;

  _showsFooter = true;
  _footerAnimatedValue = Animated.createValue(0);
  _footerAnimation: ?Animated.Animation;
  _footerAnimationStyle: Animated.Style;
  _footerRef: ?React.Node;

  _isLoginButtonActive = false;
  _loginButtonAnimatedValue = Animated.createValue(0);
  _loginButtonAnimation: ?Animated.Animation;
  _loginButtonAnimationStyle: Animated.Style;

  constructor(props: Props) {
    super(props);

    if (props.loginState === 'failed') {
      this._shouldResetLoginError = true;
    }

    this._footerAnimationStyle = Styles.createAnimatedViewStyle({
      transform: [{ translateY: this._footerAnimatedValue }],
    });

    this._loginButtonAnimationStyle = Styles.createAnimatedViewStyle({
      backgroundColor: Animated.interpolate(
        this._loginButtonAnimatedValue,
        [0.0, 1.0],
        [colors.white, colors.green],
      ),
    });
  }

  componentDidUpdate(prevProps: Props, _prevState: State) {
    if (
      this.props.loginState !== prevProps.loginState &&
      this.props.loginState === 'failed' &&
      !this._shouldResetLoginError
    ) {
      this._shouldResetLoginError = true;

      // focus on login field when failed to log in
      const accountInput = this._accountInput;
      if (accountInput) {
        accountInput.focus();
      }
    }

    this._setLoginButtonActive(this._shouldActivateLoginButton());
    this._setFooterVisibility(this._shouldShowFooter());
  }

  render() {
    return (
      <Layout>
        <Header>
          <Brand />
          <SettingsBarButton onPress={this.props.openSettings} />
        </Header>
        <Container>
          <View style={styles.login_form}>
            {this._getStatusIcon()}
            <Text style={styles.title}>{this._formTitle()}</Text>

            {this._shouldShowLoginForm() && <View>{this._createLoginForm()}</View>}
          </View>

          <Animated.View
            ref={(ref) => {
              this._footerRef = ref;
            }}
            style={[styles.login_footer, this._footerAnimationStyle]}
            testName={'footerVisibility ' + this._shouldShowFooter().toString()}>
            {this._createFooter()}
          </Animated.View>
        </Container>
      </Layout>
    );
  }

  _onCreateAccount = () => this.props.openExternalLink('createAccount');

  _onFocus = () => {
    this.setState({ isActive: true });
  };

  _onBlur = (e) => {
    const relatedTarget = e.relatedTarget;

    // restore focus if click happened within dropdown
    if (relatedTarget) {
      e.target.focus();
      return;
    }

    this.setState({ isActive: false });
  };

  async _setLoginButtonActive(isActive: boolean) {
    if (this._isLoginButtonActive === isActive) {
      return;
    }

    const animation = Animated.timing(this._loginButtonAnimatedValue, {
      toValue: isActive ? 1 : 0,
      easing: Animated.Easing.Linear(),
      duration: 250,
    });

    const oldAnimation = this._loginButtonAnimation;
    if (oldAnimation) {
      oldAnimation.stop();
    }

    animation.start();

    this._loginButtonAnimation = animation;
    this._isLoginButtonActive = isActive;
  }

  async _setFooterVisibility(show: boolean) {
    if (this._showsFooter === show) {
      return;
    }

    this._showsFooter = show;

    const layout = await UserInterface.measureLayoutRelativeToWindow(this._footerRef);
    const value = show ? 0 : layout.height;

    const animation = Animated.timing(this._footerAnimatedValue, {
      toValue: value,
      easing: Animated.Easing.InOut(),
      duration: 250,
    });

    const oldAnimation = this._footerAnimation;
    if (oldAnimation) {
      oldAnimation.stop();
    }

    animation.start();

    this._footerAnimation = animation;
  }

  _onLogin = () => {
    const accountToken = this.props.accountToken;
    if (accountToken && accountToken.length >= MIN_ACCOUNT_TOKEN_LENGTH) {
      this.props.login(accountToken);
    }
  };

  _onInputChange = (value: string) => {
    // reset error when user types in the new account number
    if (this._shouldResetLoginError) {
      this._shouldResetLoginError = false;
      this.props.resetLoginError();
    }

    this.props.updateAccountToken(value);
  };

  _formTitle() {
    switch (this.props.loginState) {
      case 'logging in':
        return 'Logging in...';
      case 'failed':
        return 'Login failed';
      case 'ok':
        return 'Login successful';
      default:
        return 'Login';
    }
  }

  _formSubtitle() {
    const { loginState, loginError } = this.props;
    switch (loginState) {
      case 'failed':
        return (loginError && loginError.message) || 'Unknown error';
      case 'logging in':
        return 'Checking account number';
      default:
        return 'Enter your account number';
    }
  }

  _getStatusIcon() {
    const statusIconPath = this._getStatusIconPath();
    return (
      <View style={styles.status_icon}>
        {statusIconPath ? <Img source={statusIconPath} height={48} width={48} alt="" /> : null}
      </View>
    );
  }

  _getStatusIconPath(): ?string {
    switch (this.props.loginState) {
      case 'logging in':
        return 'icon-spinner';
      case 'failed':
        return 'icon-fail';
      case 'ok':
        return 'icon-success';
      default:
        return undefined;
    }
  }

  _accountInputGroupStyles(): Array<Object> {
    const classes = [styles.account_input_group];
    if (this.state.isActive) {
      classes.push(styles.account_input_group__active);
    }

    switch (this.props.loginState) {
      case 'logging in':
        classes.push(styles.account_input_group__inactive);
        break;
      case 'failed':
        classes.push(styles.account_input_group__error);
        break;
    }

    return classes;
  }

  _accountInputButtonStyles(): Array<Object> {
    const classes = [styles.input_button];

    if (this.props.loginState === 'logging in') {
      classes.push(styles.input_button__invisible);
    }

    classes.push(this._loginButtonAnimationStyle);

    return classes;
  }

  _accountInputArrowStyles(): Array<Object> {
    const { accountToken, loginState } = this.props;
    const classes = [styles.input_arrow];

    if (accountToken && accountToken.length >= MIN_ACCOUNT_TOKEN_LENGTH) {
      classes.push(styles.input_arrow__active);
    }

    if (loginState === 'logging in') {
      classes.push(styles.input_arrow__invisible);
    }

    return classes;
  }

  _shouldActivateLoginButton() {
    const { accountToken } = this.props;
    return accountToken && accountToken.length >= MIN_ACCOUNT_TOKEN_LENGTH;
  }

  _shouldEnableAccountInput() {
    // enable account input always except when "logging in"
    return this.props.loginState !== 'logging in';
  }

  _shouldShowAccountHistory() {
    return (
      this._shouldEnableAccountInput() &&
      this.state.isActive &&
      this.props.accountHistory.length > 0
    );
  }

  _shouldShowLoginForm() {
    return this.props.loginState !== 'ok';
  }

  _shouldShowFooter() {
    return (
      (this.props.loginState === 'none' || this.props.loginState === 'failed') &&
      !this._shouldShowAccountHistory()
    );
  }

  _onSelectAccountFromHistory = (accountToken) => {
    this.props.updateAccountToken(accountToken);
    this.props.login(accountToken);
  };

  _onRemoveAccountFromHistory = (accountToken) => {
    this._removeAccountFromHistory(accountToken);
  };

  async _removeAccountFromHistory(accountToken: AccountToken) {
    try {
      await this.props.removeAccountTokenFromHistory(accountToken);

      // TODO: Remove account from memory
    } catch (error) {
      // TODO: Show error
    }
  }

  _createLoginForm() {
    return (
      <View>
        <Text style={styles.subtitle}>{this._formSubtitle()}</Text>
        <View style={this._accountInputGroupStyles()}>
          <View style={styles.account_input_backdrop}>
            <AccountInput
              style={styles.account_input_textfield}
              type="text"
              placeholder="0000 0000 0000 0000"
              placeholderTextColor={colors.blue40}
              onFocus={this._onFocus}
              onBlur={this._onBlur}
              onChange={this._onInputChange}
              onEnter={this._onLogin}
              value={this.props.accountToken || ''}
              editable={this._shouldEnableAccountInput()}
              autoFocus={true}
              ref={(ref) => (this._accountInput = ref)}
              testName="AccountInput"
            />
            <Animated.View
              style={this._accountInputButtonStyles()}
              onPress={this._onLogin}
              testName="account-input-button">
              <Img
                style={this._accountInputArrowStyles()}
                source="icon-arrow"
                height={16}
                width={24}
                tintColor="currentColor"
              />
            </Animated.View>
          </View>
          <Accordion height={this._shouldShowAccountHistory() ? 'auto' : 0}>
            {
              <AccountDropdown
                items={this.props.accountHistory.slice().reverse()}
                onSelect={this._onSelectAccountFromHistory}
                onRemove={this._onRemoveAccountFromHistory}
              />
            }
          </Accordion>
        </View>
      </View>
    );
  }

  _createFooter() {
    return (
      <View>
        <Text style={styles.login_footer__prompt}>{"Don't have an account number?"}</Text>
        <AppButton.BlueButton onPress={this._onCreateAccount}>
          <AppButton.Label>Create account</AppButton.Label>
          <Img source="icon-extLink" height={16} width={16} />
        </AppButton.BlueButton>
      </View>
    );
  }
}

type AccountDropdownProps = {
  items: Array<AccountToken>,
  onSelect: (value: AccountToken) => void,
  onRemove: (value: AccountToken) => void,
};

class AccountDropdown extends React.Component<AccountDropdownProps> {
  render() {
    const uniqueItems = [...new Set(this.props.items)];
    return (
      <View>
        {uniqueItems.map((token) => (
          <AccountDropdownItem
            key={token}
            value={token}
            label={formatAccount(token)}
            onSelect={this.props.onSelect}
            onRemove={this.props.onRemove}
          />
        ))}
      </View>
    );
  }
}

type AccountDropdownItemProps = {
  label: string,
  value: AccountToken,
  onRemove: (value: AccountToken) => void,
  onSelect: (value: AccountToken) => void,
};

class AccountDropdownItem extends React.Component<AccountDropdownItemProps> {
  render() {
    return (
      <View>
        <View style={styles.account_dropdown__spacer} />
        <Cell.CellButton
          style={styles.account_dropdown__item}
          cellHoverStyle={styles.account_dropdown__item_hover}>
          <Cell.Label
            style={styles.account_dropdown__label}
            cellHoverStyle={styles.account_dropdown__label_hover}
            onPress={() => this.props.onSelect(this.props.value)}>
            {this.props.label}
          </Cell.Label>
          <Img
            style={styles.account_dropdown__remove}
            cellHoverStyle={styles.account_dropdown__remove_cell_hover}
            hoverStyle={styles.account_dropdown__remove_hover}
            source="icon-close-sml"
            height={16}
            width={16}
            onPress={() => this.props.onRemove(this.props.value)}
          />
        </Cell.CellButton>
      </View>
    );
  }
}

// TODO: DRY
function formatAccount(val: string) {
  // display number altogether when longer than 12
  if (val.length > 12) {
    return val;
  } else {
    // display quartets
    return val.replace(/([0-9]{4})/g, '$1 ').trim();
  }
}
