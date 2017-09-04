// @flow
import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';
import AccountInput from './AccountInput';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';
import LoginArrowSVG from '../assets/images/icon-arrow.svg';

import type { AccountReduxState, LoginState } from '../redux/account/reducers';

export type LoginPropTypes = {
  account: AccountReduxState,
  onLogin: (accountToken: string) => void,
  onSettings: ?(() => void),
  onFirstChangeAfterFailure: () => void,
  onExternalLink: (type: string) => void,
};

export default class Login extends Component {
  props: LoginPropTypes;
  state = {
    notifyOnFirstChangeAfterFailure: false,
    isActive: false,
    unsubmittedAccountToken: '',
  };

  onCreateAccount = () => this.props.onExternalLink('createAccount');
  onFocus = () => this.setState({ isActive: true });
  onBlur = () => this.setState({ isActive: false });
  onLogin = () => {
    const accountToken = this.state.unsubmittedAccountToken;
    if(accountToken && accountToken.length > 0) {
      this.props.onLogin(accountToken);
      this.setState({
        unsubmittedAccountToken: '',
      });
    }
  }

  onInputChange = (val: string) => {
    // notify delegate on first change after login failure
    if(this.state.notifyOnFirstChangeAfterFailure) {
      this.setState({ notifyOnFirstChangeAfterFailure: false });
      this.props.onFirstChangeAfterFailure();
    }
    this.setState({
      unsubmittedAccountToken: val,
    });
  }

  formTitle(s: LoginState): string {
    switch(s) {
    case 'logging in': return 'Logging in...';
    case 'failed': return 'Login failed';
    case 'ok': return 'Login successful';
    default: return 'Login';
    }
  }

  formSubtitle(s: LoginState, e: ?Error): string {
    switch(s) {
    case 'failed':  return (e && e.message) || 'Unknown error';
    case 'logging in': return 'Checking account number';
    default: return 'Enter your account number';
    }
  }

  inputWrapClass(s: LoginState): string {
    const classes = ['login-form__input-wrap'];

    if(this.state.isActive) {
      classes.push('login-form__input-wrap--active');
    }

    switch(s) {
    case 'logging in':
      classes.push('login-form__input-wrap--inactive');
      break;
    case 'failed':
      classes.push('login-form__input-wrap--error');
      break;
    }

    return classes.join(' ');
  }

  submitClass(s: LoginState, accountToken: ?string): string {
    const classes = ['login-form__submit'];

    if(accountToken && accountToken.length > 0) {
      classes.push('login-form__submit--active');
    }

    if(s === 'logging in') {
      classes.push('login-form__submit--invisible');
    }

    return classes.join(' ');
  }

  componentWillReceiveProps(nextProps: LoginPropTypes) {
    const prev = this.props.account || {};
    const next = nextProps.account || {};

    if(prev.status !== next.status && next.status === 'failed') {
      this.setState({ notifyOnFirstChangeAfterFailure: true });
    }
  }

  render(): React.Element<*> {
    const { status } = this.props.account;
    const title = this.formTitle(status);

    const shouldShowLoginForm = status !== 'ok';
    const shouldShowFooter = status === 'none' || status === 'failed';

    const statusIcon = this._getStatusIcon();

    const loginFormClass = shouldShowLoginForm ? '' : 'login-form__fields--invisible';
    const loginForm = this._createLoginForm();

    const footerClass = shouldShowFooter ? '' : 'login-footer--invisible';
    const footer = this._createFooter();

    return (
      <Layout>
        <Header showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
          <div className="login">
            <div className="login-form">
              { statusIcon }

              <div className="login-form__title">{ title }</div>

              <div className={ 'login-form__fields ' + loginFormClass }>
                { loginForm }
              </div>
            </div>

            <div className={ 'login-footer ' + footerClass }>
              { footer }
            </div>
          </div>
        </Container>
      </Layout>
    );
  }

  _getStatusIcon(): React.Element<*> {
    const statusIconPath = this._getStatusIconPath();

    return <div className="login-form__status-icon">
      <img src={ statusIconPath } alt="" />
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

  _createLoginForm(): React.Element<*> {
    const { status, error } = this.props.account;
    const accountToken = status === 'logging in'
      ? this.props.account.accountToken
      : this.state.unsubmittedAccountToken;

    const inputDisabled = status === 'logging in';

    const subtitle = this.formSubtitle(status, error);

    const inputWrapClass = this.inputWrapClass(status);
    const submitClass = this.submitClass(status, accountToken);

    const autoFocusRef = input => {
      if(status === 'failed' && input) {
        input.focus();
      }
    };

    return <div>
      <div className="login-form__subtitle">{ subtitle }</div>
      <div className={ inputWrapClass }>
        <AccountInput className="login-form__input-field"
          type="text"
          placeholder="e.g 0000 0000 0000"
          onFocus={ this.onFocus }
          onBlur={ this.onBlur }
          onChange={ this.onInputChange }
          onEnter={ this.onLogin }
          value={ accountToken || '' }
          disabled={ inputDisabled }
          autoFocus={ true }
          ref={ autoFocusRef } />
        <button className={ submitClass } onClick={ this.onLogin }>
          <LoginArrowSVG className="login-form__submit-icon" />
        </button>
      </div>
    </div>;
  }

  _createFooter(): React.Element<*> {
    return <div>
      <div className="login-footer__prompt">{ 'Don\'t have an account number?' }</div>
      <button className="button button--primary" onClick={ this.onCreateAccount }>
        <span className="button-label">Create account</span>
        <ExternalLinkSVG className="button-icon button-icon--16" />
      </button>
    </div>;
  }
}

