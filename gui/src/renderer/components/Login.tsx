import * as React from 'react';
import { Animated, Component, Styles, Text, TextInput, Types, UserInterface, View } from 'reactxp';
import { colors } from '../../config.json';
import consumePromise from '../../shared/promise';
import { messages } from '../../shared/gettext';
import { formatAccountToken } from '../lib/account';
import Accordion from './Accordion';
import * as AppButton from './AppButton';
import { Brand, HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Header, Layout } from './Layout';
import styles, {
  AccountDropdownItemButton,
  AccountDropdownItemButtonLabel,
  AccountDropdownRemoveIcon,
  InputSubmitIcon,
} from './LoginStyles';

import { AccountToken } from '../../shared/daemon-rpc-types';
import { LoginState } from '../redux/account/reducers';

interface IProps {
  accountToken?: AccountToken;
  accountHistory: AccountToken[];
  loginState: LoginState;
  openExternalLink: (type: string) => void;
  login: (accountToken: AccountToken) => void;
  resetLoginError: () => void;
  updateAccountToken: (accountToken: AccountToken) => void;
  removeAccountTokenFromHistory: (accountToken: AccountToken) => Promise<void>;
  createNewAccount: () => void;
}

interface IState {
  isActive: boolean;
}

const MIN_ACCOUNT_TOKEN_LENGTH = 10;

export default class Login extends Component<IProps, IState> {
  public state: IState = {
    isActive: true,
  };

  private accountInput = React.createRef<TextInput>();
  private shouldResetLoginError = false;

  private showsFooter = true;
  private footerAnimatedValue = Animated.createValue(0);
  private footerAnimation?: Types.Animated.CompositeAnimation;
  private footerAnimationStyle: Types.AnimatedViewStyleRuleSet;
  private footerRef = React.createRef<Animated.View>();

  private isLoginButtonActive = false;
  private loginButtonAnimatedValue = Animated.createValue(0);
  private loginButtonAnimation?: Types.Animated.CompositeAnimation;
  private loginButtonAnimationStyle: Types.AnimatedViewStyleRuleSet;

  constructor(props: IProps) {
    super(props);

    if (props.loginState.type === 'failed') {
      this.shouldResetLoginError = true;
    }

    this.footerAnimationStyle = Styles.createAnimatedViewStyle({
      transform: [{ translateY: this.footerAnimatedValue }],
    });

    this.loginButtonAnimationStyle = Styles.createAnimatedViewStyle({
      backgroundColor: Animated.interpolate(
        this.loginButtonAnimatedValue,
        [0.0, 1.0],
        [colors.white, colors.green],
      ),
    });
  }

  public componentDidMount() {
    consumePromise(this.setFooterVisibility(this.shouldShowFooter()));
  }

  public componentDidUpdate(prevProps: IProps, _prevState: IState) {
    if (
      this.props.loginState.type !== prevProps.loginState.type &&
      this.props.loginState.type === 'failed' &&
      !this.shouldResetLoginError
    ) {
      this.shouldResetLoginError = true;

      // focus on login field when failed to log in
      const accountInput = this.accountInput.current;
      if (accountInput) {
        accountInput.focus();
      }
    }

    this.setLoginButtonActive(this.shouldActivateLoginButton());
    consumePromise(this.setFooterVisibility(this.shouldShowFooter()));
  }

  public render() {
    return (
      <Layout>
        <Header>
          <Brand />
          <HeaderBarSettingsButton />
        </Header>
        <Container>
          <View style={styles.login_form}>
            {this.getStatusIcon()}
            <Text style={styles.title}>{this.formTitle()}</Text>

            {this.createLoginForm()}
          </View>

          <Animated.View
            ref={this.footerRef}
            style={[styles.login_footer, this.footerAnimationStyle]}>
            {this.createFooter()}
          </Animated.View>
        </Container>
      </Layout>
    );
  }

  private onFocus = () => {
    this.setState({ isActive: true });
  };

  private onBlur = (e: Types.SyntheticEvent) => {
    // TOOD: relatedTarget is not exposed by ReactXP and may not work on non-web platforms.
    // Find a workaround.
    // @ts-ignore
    const relatedTarget = e.relatedTarget;

    // restore focus if click happened within dropdown
    if (relatedTarget) {
      if (this.accountInput.current) {
        this.accountInput.current.focus();
      }
      return;
    }

    this.setState({ isActive: false });
  };

  private setLoginButtonActive(isActive: boolean) {
    if (this.isLoginButtonActive === isActive) {
      return;
    }

    const animation = Animated.timing(this.loginButtonAnimatedValue, {
      toValue: isActive ? 1 : 0,
      easing: Animated.Easing.Linear(),
      duration: 250,
    });

    const oldAnimation = this.loginButtonAnimation;
    if (oldAnimation) {
      oldAnimation.stop();
    }

    animation.start();

    this.loginButtonAnimation = animation;
    this.isLoginButtonActive = isActive;
  }

  private async setFooterVisibility(show: boolean) {
    if (this.showsFooter === show || !this.footerRef.current) {
      return;
    }

    this.showsFooter = show;

    const layout = await UserInterface.measureLayoutRelativeToWindow(this.footerRef.current);
    const value = show ? 0 : layout.height;

    const animation = Animated.timing(this.footerAnimatedValue, {
      toValue: value,
      easing: Animated.Easing.InOut(),
      duration: 250,
    });

    const oldAnimation = this.footerAnimation;
    if (oldAnimation) {
      oldAnimation.stop();
    }

    animation.start();

    this.footerAnimation = animation;
  }

  private onSubmit = () => {
    const accountToken = this.props.accountToken;
    if (accountToken && accountToken.length >= MIN_ACCOUNT_TOKEN_LENGTH) {
      this.props.login(accountToken);
    }
  };

  private onInputChange = (value: string) => {
    // reset error when user types in the new account number
    if (this.shouldResetLoginError) {
      this.shouldResetLoginError = false;
      this.props.resetLoginError();
    }

    const accountToken = value.replace(/[^0-9]/g, '');

    this.props.updateAccountToken(accountToken);
  };

  private formTitle() {
    switch (this.props.loginState.type) {
      case 'logging in':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Logging in...')
          : messages.pgettext('login-view', 'Creating account...');
      case 'failed':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Login failed')
          : messages.pgettext('login-view', 'Error');
      case 'ok':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Logged in')
          : messages.pgettext('login-view', 'Account created');
      default:
        return messages.pgettext('login-view', 'Login');
    }
  }

  private formSubtitle() {
    switch (this.props.loginState.type) {
      case 'failed':
        return this.props.loginState.method === 'existing_account'
          ? this.props.loginState.error.message || messages.pgettext('login-view', 'Unknown error')
          : messages.pgettext('login-view', 'Failed to create account');
      case 'logging in':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Checking account number')
          : messages.pgettext('login-view', 'Please wait');
      case 'ok':
        return this.props.loginState.method === 'existing_account'
          ? messages.pgettext('login-view', 'Valid account number')
          : messages.pgettext('login-view', 'Logged in');
      default:
        return messages.pgettext('login-view', 'Enter your account number');
    }
  }

  private getStatusIcon() {
    const statusIconPath = this.getStatusIconPath();
    return (
      <View style={styles.status_icon}>
        {statusIconPath ? <ImageView source={statusIconPath} height={48} width={48} /> : null}
      </View>
    );
  }

  private getStatusIconPath(): string | undefined {
    switch (this.props.loginState.type) {
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

  private accountInputGroupStyles(): Types.ViewStyleRuleSet[] {
    const classes = [styles.account_input_group];
    if (this.state.isActive) {
      classes.push(styles.account_input_group__active);
    }

    if (!this.allowInteraction()) {
      classes.push(styles.account_input_group__inactive);
    } else if (
      this.props.loginState.type === 'failed' &&
      this.props.loginState.method === 'existing_account'
    ) {
      classes.push(styles.account_input_group__error);
    }

    return classes;
  }

  private accountInputButtonStyles() {
    const classes: Array<
      Types.StyleRuleSet<Types.AnimatedViewStyle> | Types.StyleRuleSet<Types.ViewStyle>
    > = [styles.input_button];

    if (!this.allowInteraction()) {
      classes.push(styles.input_button__invisible);
    }

    classes.push(this.loginButtonAnimationStyle);

    return classes;
  }

  private allowInteraction() {
    return this.props.loginState.type !== 'logging in' && this.props.loginState.type !== 'ok';
  }

  private shouldActivateLoginButton(): boolean {
    const { accountToken } = this.props;
    if (accountToken && accountToken.length >= MIN_ACCOUNT_TOKEN_LENGTH) {
      return true;
    }
    return false;
  }

  private shouldShowAccountHistory() {
    return this.allowInteraction() && this.state.isActive && this.props.accountHistory.length > 0;
  }

  private shouldShowFooter() {
    return (
      (this.props.loginState.type === 'none' || this.props.loginState.type === 'failed') &&
      !this.shouldShowAccountHistory()
    );
  }

  private onSelectAccountFromHistory = (accountToken: string) => {
    this.props.updateAccountToken(accountToken);
    this.props.login(accountToken);
  };

  private onRemoveAccountFromHistory = (accountToken: string) => {
    consumePromise(this.removeAccountFromHistory(accountToken));
  };

  private async removeAccountFromHistory(accountToken: AccountToken) {
    try {
      await this.props.removeAccountTokenFromHistory(accountToken);

      // TODO: Remove account from memory
    } catch (error) {
      // TODO: Show error
    }
  }

  private createLoginForm() {
    return (
      <View>
        <Text style={styles.subtitle}>{this.formSubtitle()}</Text>
        <View style={this.accountInputGroupStyles()}>
          <View style={styles.account_input_backdrop}>
            <TextInput
              style={styles.account_input_textfield}
              placeholder="0000 0000 0000 0000"
              placeholderTextColor={colors.blue40}
              value={this.props.accountToken || ''}
              autoCorrect={false}
              editable={this.allowInteraction()}
              onFocus={this.onFocus}
              onBlur={this.onBlur}
              onChangeText={this.onInputChange}
              onSubmitEditing={this.onSubmit}
              returnKeyType="done"
              keyboardType="numeric"
              autoFocus={true}
              ref={this.accountInput}
            />
            <Animated.View style={this.accountInputButtonStyles()} onPress={this.onSubmit}>
              <InputSubmitIcon
                visible={this.props.loginState.type !== 'logging in'}
                source="icon-arrow"
                height={16}
                width={24}
                tintColor="rgb(255, 255, 255)"
              />
            </Animated.View>
          </View>
          <Accordion expanded={this.shouldShowAccountHistory()}>
            {
              <AccountDropdown
                items={this.props.accountHistory.slice().reverse()}
                onSelect={this.onSelectAccountFromHistory}
                onRemove={this.onRemoveAccountFromHistory}
              />
            }
          </Accordion>
        </View>
      </View>
    );
  }

  private createFooter() {
    return (
      <View>
        <Text style={styles.login_footer__prompt}>
          {messages.pgettext('login-view', "Don't have an account number?")}
        </Text>
        <AppButton.BlueButton
          onClick={this.props.createNewAccount}
          disabled={!this.allowInteraction()}>
          {messages.pgettext('login-view', 'Create account')}
        </AppButton.BlueButton>
      </View>
    );
  }
}

interface IAccountDropdownProps {
  items: AccountToken[];
  onSelect: (value: AccountToken) => void;
  onRemove: (value: AccountToken) => void;
}

class AccountDropdown extends Component<IAccountDropdownProps> {
  public render() {
    const uniqueItems = [...new Set(this.props.items)];
    return (
      <View>
        {uniqueItems.map((token) => {
          const label = formatAccountToken(token);
          return (
            <AccountDropdownItem
              key={token}
              value={token}
              label={label}
              onSelect={this.props.onSelect}
              onRemove={this.props.onRemove}
            />
          );
        })}
      </View>
    );
  }
}

interface IAccountDropdownItemProps {
  label: string;
  value: AccountToken;
  onRemove: (value: AccountToken) => void;
  onSelect: (value: AccountToken) => void;
}

class AccountDropdownItem extends Component<IAccountDropdownItemProps> {
  public render() {
    return (
      <View>
        <View style={styles.account_dropdown__spacer} />
        <AccountDropdownItemButton>
          <AccountDropdownItemButtonLabel onClick={this.handleSelect}>
            {this.props.label}
          </AccountDropdownItemButtonLabel>
          <AccountDropdownRemoveIcon
            tintColor={colors.blue40}
            tintHoverColor={colors.blue}
            source="icon-close-sml"
            height={16}
            width={16}
            onClick={this.handleRemove}
          />
        </AccountDropdownItemButton>
      </View>
    );
  }

  private handleSelect = () => {
    this.props.onSelect(this.props.value);
  };

  private handleRemove = () => {
    this.props.onRemove(this.props.value);
  };
}
