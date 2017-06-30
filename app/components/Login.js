// @flow
import React, { Component } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import AccountInput from './AccountInput';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';
import LoginArrowSVG from '../assets/images/icon-arrow.svg';

import type { AccountReduxState, LoginState } from '../redux/account/reducers';

export type LoginPropTypes = {
  account: AccountReduxState,
  onLogin: (accountNumber: string) => void,
  onSettings: ?(() => void),
  onChange: (input: string) => void,
  onFirstChangeAfterFailure: () => void,
  onExternalLink: (type: string) => void,
};

export default class Login extends Component {
  props: LoginPropTypes;
  state = {
    notifyOnFirstChangeAfterFailure: false,
    isActive: false
  }

  onCreateAccount = () => this.props.onExternalLink('createAccount');
  onFocus = () => this.setState({ isActive: true });
  onBlur = () => this.setState({ isActive: false });
  onLogin = () => {
    const { accountNumber } = this.props.account;
    if(accountNumber && accountNumber.length > 0) {
      this.props.onLogin(accountNumber);
    }
  }

  onInputChange = (val: string) => {
    // notify delegate on first change after login failure
    if(this.state.notifyOnFirstChangeAfterFailure) {
      this.setState({ notifyOnFirstChangeAfterFailure: false });
      this.props.onFirstChangeAfterFailure();
    }
    this.props.onChange(val);
  }

  formTitle(s: LoginState): string {
    switch(s) {
    case 'connecting': return 'Logging in...';
    case 'failed': return 'Login failed';
    case 'ok': return 'Login successful';
    default: return 'Login';
    }
  }

  formSubtitle(s: LoginState, e: ?Error): string {
    switch(s) {
    case 'failed':  return (e && e.message) || 'Unknown error';
    case 'connecting': return 'Checking account number';
    default: return 'Enter your account number';
    }
  }

  inputWrapClass(s: LoginState): string {
    const classes = ['login-form__input-wrap'];

    if(this.state.isActive) {
      classes.push('login-form__input-wrap--active');
    }

    switch(s) {
    case 'connecting':
      classes.push('login-form__input-wrap--inactive');
      break;
    case 'failed':
      classes.push('login-form__input-wrap--error');
      break;
    }

    return classes.join(' ');
  }

  footerClass(s: LoginState): string {
    const classes = ['login-footer'];
    switch(s) {
    case 'ok':
    case 'connecting':
      classes.push('login-footer--invisible');
      break;
    }
    return classes.join(' ');
  }

  submitClass(s: LoginState, accountNumber: ?string): string {
    const classes = ['login-form__submit'];

    if(accountNumber && accountNumber.length > 0) {
      classes.push('login-form__submit--active');
    }

    if(s === 'connecting') {
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
    const { accountNumber, status, error } = this.props.account;
    const title = this.formTitle(status);
    const subtitle = this.formSubtitle(status, error);

    let isConnecting = false;
    let isFailed = false;
    let isLoggedIn = false;
    switch(status) {
    case 'connecting': isConnecting = true; break;
    case 'failed': isFailed = true; break;
    case 'ok': isLoggedIn = true; break;
    }

    const inputWrapClass = this.inputWrapClass(status);
    const footerClass = this.footerClass(status);
    const submitClass = this.submitClass(status, accountNumber);

    const autoFocusRef = input => {
      if(isFailed && input) {
        input.focus();
      }
    };

    return (
      <Layout>
        <Header showSettings={ true } onSettings={ this.props.onSettings } />
        <Container>
          <div className="login">
            <div className="login-form">
              { /* show spinner when connecting */ }
              <If condition={ isConnecting }>
                <Then>
                  <div className="login-form__status-icon">
                    <img src="./assets/images/icon-spinner.svg" alt="" />
                  </div>
                </Then>
              </If>

              { /* show error icon when failed */ }
              <If condition={ isFailed }>
                <Then>
                  <div className="login-form__status-icon">
                    <img src="./assets/images/icon-fail.svg" alt="" />
                  </div>
                </Then>
              </If>

              { /* show tick when logged in */ }
              <If condition={ isLoggedIn }>
                <Then>
                  <div className="login-form__status-icon">
                    <img src="./assets/images/icon-success.svg" alt="" />
                  </div>
                </Then>
              </If>

              <div className="login-form__title">{ title }</div>
              <div className={ 'login-form__fields' + (isLoggedIn ? ' login-form__fields--invisible' : '') }>
                <div className="login-form__subtitle">{ subtitle }</div>
                <div className={ inputWrapClass }>
                  <AccountInput className="login-form__input-field"
                    type="text"
                    placeholder="e.g 0000 0000 0000"
                    onFocus={ this.onFocus }
                    onBlur={ this.onBlur }
                    onChange={ this.onInputChange }
                    onEnter={ this.onLogin }
                    value={ accountNumber || '' }
                    disabled={ isConnecting }
                    autoFocus={ true }
                    ref={ autoFocusRef } />
                  <button className={ submitClass } onClick={ this.onLogin }>
                    <LoginArrowSVG className="login-form__submit-icon" />
                  </button>
                </div>
              </div>
            </div>
            <div className={ footerClass }>
              <div className="login-footer__prompt">{ 'Don\'t have an account number?' }</div>
              <button className="button button--primary" onClick={ this.onCreateAccount }>
                <span className="button-label">Create account</span>
                <ExternalLinkSVG className="button-icon button-icon--16" />
              </button>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
