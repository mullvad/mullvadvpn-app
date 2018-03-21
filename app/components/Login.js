// @flow
import * as React from 'react';
import { Layout, Container, Header } from './Layout';
import AccountInput from './AccountInput';
import Accordion from './Accordion';
import { formatAccount } from '../lib/formatters';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';
import LoginArrowSVG from '../assets/images/icon-arrow.svg';
import RemoveAccountSVG from '../assets/images/icon-close-sml.svg';

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
  isActive: boolean
};

export default class Login extends React.Component<LoginPropTypes, State> {
  state = {
    notifyOnFirstChangeAfterFailure: false,
    isActive: false,
  };

  constructor(props: LoginPropTypes) {
    super(props);
    if(props.account.status === 'failed') {
      this.state.notifyOnFirstChangeAfterFailure = true;
    }
  }

  componentWillReceiveProps(nextProps: LoginPropTypes) {
    const prev = this.props.account || {};
    const next = nextProps.account || {};

    if(prev.status !== next.status && next.status === 'failed') {
      this.setState({ notifyOnFirstChangeAfterFailure: true });
    }
  }

  render() {
    const footerClass = this._shouldShowFooter() ? '' : 'login-footer--invisible';
    return (
      <Layout>
        <Header showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
          <div className="login">
            <div className="login-form">
              { this._getStatusIcon() }

              <div className="login-form__title">{ this._formTitle() }</div>

              {this._shouldShowLoginForm() && <div className='login-form__fields'>
                { this._createLoginForm() }
              </div>}
            </div>

            <div className={ 'login-footer ' + footerClass }>
              { this._createFooter() }
            </div>
          </div>
        </Container>
      </Layout>
    );
  }

  _onCreateAccount = () => this.props.onExternalLink('createAccount');
  _onFocus = () => this.setState({ isActive: true });
  _onBlur = (e) => {
    const relatedTarget = e.relatedTarget;

    // restore focus if click happened within dropdown
    if(relatedTarget && this._isWithinDropdown(relatedTarget)) {
      e.target.focus();
      return;
    }

    this.setState({ isActive: false });
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
    return <div className="login-form__status-icon">
      { statusIconPath ?
        <img src={ statusIconPath } alt="" /> :
        null }
    </div>;
  }

  _getStatusIconPath(): ?string {
    switch(this.props.account.status) {
    case 'logging in':
      return './assets/images/icon-spinner.svg';
    case 'failed':
      return './assets/images/icon-fail.svg';
    case 'ok':
      return './assets/images/icon-success.svg';
    default:
      return undefined;
    }
  }

  _accountInputGroupClass(): string {
    const classes = ['login-form__account-input-group'];
    if(this.state.isActive) {
      classes.push('login-form__account-input-group--active');
    }

    switch(this.props.account.status) {
    case 'logging in':
      classes.push('login-form__account-input-group--inactive');
      break;
    case 'failed':
      classes.push('login-form__account-input-group--error');
      break;
    }

    return classes.join(' ');
  }

  _accountInputButtonClass(): string {
    const { accountToken, status } = this.props.account;
    const classes = ['login-form__account-input-button'];

    if(accountToken && accountToken.length > 0) {
      classes.push('login-form__account-input-button--active');
    }

    if(status === 'logging in') {
      classes.push('login-form__account-input-button--invisible');
    }

    return classes.join(' ');
  }

  _shouldEnableAccountInput() {
    // enable account input always except when "logging in"
    return this.props.account.status !== 'logging in';
  }

  _shouldShowAccountHistory() {
    return this._shouldEnableAccountInput() &&
      this.state.isActive &&
      this.props.account.accountHistory.length > 0;
  }

  _shouldShowLoginForm() {
    return this.props.account.status !== 'ok';
  }

  _shouldShowFooter() {
    const { status } = this.props.account;
    return (status === 'none' || status === 'failed') && !this._shouldShowAccountHistory();
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

    return <div>
      <div className="login-form__subtitle">{ this._formSubtitle() }</div>
      <div className="login-form__account-input-container">
        <div className={ this._accountInputGroupClass() }>
          <div className="login-form__account-input-backdrop">
            <AccountInput className="login-form__account-input-textfield"
              type="text"
              placeholder="e.g 0000 0000 0000"
              onFocus={ this._onFocus }
              onBlur={ this._onBlur }
              onChange={ this._onInputChange }
              onEnter={ this._onLogin }
              value={ accountToken || '' }
              disabled={ !this._shouldEnableAccountInput() }
              autoFocus={ true }
              ref={ autoFocusOnFailure } />
            <button className={ this._accountInputButtonClass() } onClick={ this._onLogin }>
              <LoginArrowSVG className="login-form__account-input-button-icon" />
            </button>
          </div>
          <Accordion height={ this._shouldShowAccountHistory() ? 'auto' : 0 }>
            <div ref={ this._onAccountDropdownContainerRef }>
              { <AccountDropdown
                items={ accountHistory.slice().reverse() }
                onSelect={ this._onSelectAccountFromHistory }
                onRemove={ this.props.onRemoveAccountTokenFromHistory } /> }
            </div>
          </Accordion>
        </div>
      </div>
    </div>;
  }

  _createFooter() {
    return <div>
      <div className="login-footer__prompt">{ 'Don\'t have an account number?' }</div>
      <button className="button button--primary" onClick={ this._onCreateAccount }>
        <span className="button-label">Create account</span>
        <ExternalLinkSVG className="button-icon button-icon--16" />
      </button>
    </div>;
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
      <div className="login-form__account-dropdown">
        { uniqueItems.map(token => (
          <AccountDropdownItem key={ token }
            value={ token }
            label={ formatAccount(token) }
            onSelect={ this.props.onSelect }
            onRemove={ this.props.onRemove } />
        )) }
      </div>
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
      <div className="login-form__account-dropdown__item">
        <button className="login-form__account-dropdown__label"
          onClick={ () => this.props.onSelect(this.props.value) }>{ this.props.label }</button>
        <button className="login-form__account-dropdown__remove"
          onClick={ () => this.props.onRemove(this.props.value) }>
          <RemoveAccountSVG />
        </button>
      </div>
    );
  }
}
