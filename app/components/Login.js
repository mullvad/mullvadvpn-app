// @flow
import * as React from 'react';
import { Component, Text, View, Animated, Styles } from 'reactxp';
import { Layout, Container, Header } from './Layout';
import AccountInput from './AccountInput';
import { formatAccount } from '../lib/formatters';
import Img from './Img';
import { Button, BlueButton, Label } from './styled';
import styles from './LoginStyles';
import { colors } from '../config';

import type { AccountReduxState } from '../redux/account/reducers';
import type { AccountToken } from '../lib/ipc-facade';

export type LoginPropTypes = {
  account: AccountReduxState,
  onLogin: (accountToken: AccountToken) => void,
  onSettings: ?(() => void),
  onFirstChangeAfterFailure: () => void,
  onExternalLink: (type: string) => void,
  onAccountTokenChange: (accountToken: AccountToken) => void,
  onRemoveAccountTokenFromHistory: (accountToken: AccountToken) => void,
};

type State = {
  notifyOnFirstChangeAfterFailure: boolean,
  isActive: boolean,
  dropdownHeight: number,
  footerHeight: number,
  animatedFooterValue: Animated.Value,
  animatedDropdownValue: Animated.Value,
  animation: Animated.CompositeAnimation,
  footerAnimationStyle: Animated.Style,
  dropdownAnimationStyle: Animated.Style,
};

export default class Login extends Component<LoginPropTypes, State> {
  state = {
    notifyOnFirstChangeAfterFailure: false,
    isActive: false,
    dropdownHeight: 0,
    footerHeight: 0,
    animatedFooterValue: Animated.createValue(0),
    animatedDropdownValue: Animated.createValue(0),
    animation: null,
    footerAnimationStyle: null,
    dropdownAnimationStyle: null,
  };

  constructor(props: LoginPropTypes) {
    super(props);
    if(props.account.status === 'failed') {
      this.state.notifyOnFirstChangeAfterFailure = true;
    }
    this.state.dropdownAnimationStyle = Styles.createAnimatedViewStyle({
      height: this.state.animatedDropdownValue
    });
    this.state.footerAnimationStyle = Styles.createAnimatedViewStyle({
      transform: [{translateY: this.state.animatedFooterValue }]
    });
  }

  componentWillReceiveProps(nextProps: LoginPropTypes) {
    const prev = this.props.account || {};
    const next = nextProps.account || {};

    if(prev.status !== next.status && next.status === 'failed') {
      this.setState({ notifyOnFirstChangeAfterFailure: true });
    }

    this._animate(nextProps);
  }

  render() {
    return (
      <Layout>
        <Header showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
          <View style={styles.login}>
            <View style={styles.login_form}>
              { this._getStatusIcon() }

              <Text style={styles.title}>{ this._formTitle() }</Text>

              {this._shouldShowLoginForm() && <View>
                { this._createLoginForm() }
              </View>}
            </View>

            <Animated.View onLayout={this._onFooterLayout} style={[styles.login_footer, this.state.footerAnimationStyle]} testName={'footerVisibility ' + this._shouldShowFooter(this.props).toString()}>
              { this._createFooter() }
            </Animated.View>
          </View>
        </Container>
      </Layout>
    );
  }

  _onCreateAccount = () => this.props.onExternalLink('createAccount')

  _onFocus = () => this.setState({ isActive: true }, () => {
    this._animate(this.props);
  })

  _onBlur = (e) => {
    const relatedTarget = e.relatedTarget;

    // restore focus if click happened within dropdown
    if(relatedTarget && this._isWithinDropdown(relatedTarget)) {
      e.target.focus();
      return;
    }

    this.setState({ isActive: false }, () => {
      this._animate(this.props);
    });
  }

  _animate = (props: LoginPropTypes) => {
    if (this.state.animation) {
      this.state.animation.stop();
    }
    const footerPosition = this._shouldShowFooter(props) ? 0 : this.state.footerHeight;
    const dropdownHeight = this._shouldShowAccountHistory(props) ? this.state.dropdownHeight : 0;
    this._setAnimation(this._getFooterAnimation(footerPosition), this._getDropdownAnimation(dropdownHeight));
  }

  _setAnimation = (footerAnimation: Animated.CompositeAnimation, dropdownAnimation: Animated.CompositeAnimation) => {
    let compositeAnimation = Animated.parallel([ footerAnimation, dropdownAnimation ]);
    this.setState({animation: compositeAnimation}, () => {
      this.state.animation.start(() => this.setState({
        animation: null
      }));
    });
  }

  _onLogin = () => {
    const accountToken = this.props.account.accountToken;
    if(accountToken && accountToken.length > 0) {
      this.props.onLogin(accountToken);
    }
  }

  _onInputChange = (value: string) => {
    // notify delegate on first change after login failure
    if(this.state.notifyOnFirstChangeAfterFailure) {
      this.setState({ notifyOnFirstChangeAfterFailure: false });
      this.props.onFirstChangeAfterFailure();
    }
    this.props.onAccountTokenChange(value);
  }

  _formTitle() {
    switch(this.props.account.status) {
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
    const { status, error } = this.props.account;
    switch(status) {
    case 'failed':
      return (error && error.message) || 'Unknown error';
    case 'logging in':
      return 'Checking account number';
    default:
      return 'Enter your account number';
    }
  }

  _getStatusIcon() {
    const statusIconPath = this._getStatusIconPath();
    return <View style={ styles.status_icon}>
      { statusIconPath ?
        <Img source={ statusIconPath } height='48' width='48' alt="" /> :
        null }
    </View>;
  }

  _getStatusIconPath(): ?string {
    switch(this.props.account.status) {
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

  _accountInputGroupClass(): Array<Object> {
    const classes = [styles.account_input_group];
    if(this.state.isActive) {
      classes.push(styles.account_input_group__active);
    }

    switch(this.props.account.status) {
    case 'logging in':
      classes.push(styles.account_input_group__inactive);
      break;
    case 'failed':
      classes.push(styles.account_input_group__error);
      break;
    }

    return classes;
  }

  _accountInputButtonClass(): Array<Object> {
    const { accountToken, status } = this.props.account;
    const classes = [styles.account_input_button];

    if(accountToken && accountToken.length > 0) {
      classes.push(styles.account_input_button__active);
    }

    if(status === 'logging in') {
      classes.push(styles.account_input_button__invisible);
    }

    return classes;
  }

  _shouldEnableAccountInput(props: LoginPropTypes) {
    // enable account input always except when "logging in"
    return props.account.status !== 'logging in';
  }

  _shouldShowAccountHistory(props: LoginPropTypes) {
    return this._shouldEnableAccountInput(props) &&
      this.state.isActive &&
      props.account.accountHistory.length > 0;
  }

  _shouldShowLoginForm() {
    return this.props.account.status !== 'ok';
  }

  _shouldShowFooter(props: LoginPropTypes) {
    const { status } = props.account;
    return (status === 'none' || status === 'failed') && !this._shouldShowAccountHistory(props);
  }

  _getFooterAnimation(toValue: number){
    return Animated.timing(this.state.animatedFooterValue, {
      toValue: toValue,
      easing: Animated.Easing.InOut(),
      duration: 250,
      useNativeDriver: true,
    });
  }

  _onFooterLayout = (layout) => {
    this.setState({footerHeight: layout.height});
  }

  _getDropdownAnimation(toValue: number){
    return Animated.timing(this.state.animatedDropdownValue, {
      toValue: toValue,
      easing: Animated.Easing.InOut(),
      duration: 250,
      useNativeDriver: true,
    });
  }

  _onDropdownLayout = (layout) => {
    this.setState({dropdownHeight: layout.height});
  }

  // returns true if DOM node is within dropdown hierarchy
  _isWithinDropdown(relatedTarget) {
    const dropdownElement = this._accountDropdownElement;
    return dropdownElement && dropdownElement.contains(relatedTarget);
  }

  // container element used for measuring the height of the accounts dropdown
  _accountDropdownElement: ?HTMLElement;
  _onAccountDropdownContainerRef = ref => this._accountDropdownElement = ref;

  _onSelectAccountFromHistory = (accountToken) => {
    this.props.onAccountTokenChange(accountToken);
    this.props.onLogin(accountToken);
  }

  _createLoginForm() {
    const { accountHistory, accountToken } = this.props.account;

    // auto-focus on account input when failed to log in
    // do not refactor this into instance method,
    // it has to be new function each time to be called on each render
    const autoFocusOnFailure = (input) => {
      if(this.props.account.status === 'failed' && input) {
        input.focus();
      }
    };

    return <View style= {styles.login}>
      <Text style={ styles.subtitle }>{ this._formSubtitle() }</Text>
      <View style={ this._accountInputGroupClass() }>
        <View style={ styles.account_input_backdrop}>
          <AccountInput style={styles.account_input_textfield}
            type="text"
            placeholder="e.g 0000 0000 0000"
            placeholderTextColor={colors.blue40}
            onFocus={ this._onFocus }
            onBlur={ this._onBlur }
            onChange={ this._onInputChange }
            onEnter={ this._onLogin }
            value={ accountToken || '' }
            disabled={ !this._shouldEnableAccountInput(this.props) }
            autoFocus={ true }
            ref={ autoFocusOnFailure }
            testName='AccountInput'/>
          <Button style={ this._accountInputButtonClass() } onPress={ this._onLogin } testName='account-input-button'>
            <Img style={[ this._accountInputButtonClass() ]} source='icon-arrow' height='16' width='24' tintColor='currentColor' />
          </Button>
        </View>
        <Animated.View style={ this.state.dropdownAnimationStyle }>
          <View onLayout={this._onDropdownLayout} ref={ this._onAccountDropdownContainerRef }>
            { <AccountDropdown
              items={ accountHistory.slice().reverse() }
              onSelect={ this._onSelectAccountFromHistory }
              onRemove={ this.props.onRemoveAccountTokenFromHistory } /> }
          </View>
        </Animated.View>
      </View>
    </View>;
  }

  _createFooter() {
    return <View>
      <Text style={ styles.login_footer__prompt}>{ 'Don\'t have an account number?' }</Text>
      <BlueButton onPress={ this._onCreateAccount }>
        <Label>Create account</Label>
        <Img source='icon-extLink' height='16' width='16' />
      </BlueButton>
    </View>;
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
        { uniqueItems.map(token => (
          <AccountDropdownItem key={ token }
            value={ token }
            label={ formatAccount(token) }
            onSelect={ this.props.onSelect }
            onRemove={ this.props.onRemove } />
        )) }
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
    return (<View>
      <View style={ styles.account_dropdown__spacer }/>
      <View style={ styles.account_dropdown__item }>
        <Button style={styles.account_dropdown__label}
          onPress={ () => this.props.onSelect(this.props.value) }>{ this.props.label }</Button>
        <Button style={styles.account_dropdown__remove}
          onPress={ () => this.props.onRemove(this.props.value) }>
          <Img source='icon-close-sml' height='16' width='16' />
        </Button>
      </View>
    </View>
    );
  }
}
