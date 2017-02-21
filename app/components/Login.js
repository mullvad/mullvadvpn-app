import { shell } from 'electron';
import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { links, LoginState } from '../constants';
import { formatAccount } from '../lib/formatters';

export default class Login extends Component {
  static propTypes = {
    user: PropTypes.object.isRequired,
    onLogin: PropTypes.func.isRequired,
    onChange: PropTypes.func.isRequired,
    onFirstChangeAfterFailure: PropTypes.func.isRequired
  };
  
  constructor(props) {
    super(props);

    this.state = { notifyOnFirstChangeAfterFailure: false };
  }
  
  componentWillReceiveProps(nextProps) {
    const prev = this.props.user || {};
    const next = nextProps.user || {};

    if(prev.status !== next.status && next.status === LoginState.failed) {
      this.setState({ notifyOnFirstChangeAfterFailure: true });
    }
  }

  handleLogin() {
    const { account } = this.props.user;
    if(account.length > 0) {
      this.props.onLogin(account);
    }
  }

  handleCreateAccount() {
    shell.openExternal(links.createAccount);
  }

  handleInputChange(e) {
    const val = e.target.value.replace(/[^0-9]/g, '');

    // notify delegate on first change after login failure
    if(this.state.notifyOnFirstChangeAfterFailure) {
      this.setState({ notifyOnFirstChangeAfterFailure: false });
      this.props.onFirstChangeAfterFailure();
    }

    this.props.onChange(val);
  }

  handleInputKeyUp(e) {
    if(e.which === 13) {
      this.handleLogin();
    }
  }

  formTitle(s) {
    switch(s) {
    case LoginState.connecting: return 'Logging in...';
    case LoginState.failed: return 'Login failed';
    case LoginState.ok: return 'Login successful';
    default: return 'Login';
    }
  }

  formSubtitle(s, e) {
    switch(s) {
    case LoginState.failed: return e.message;
    case LoginState.connecting: return 'Checking account number';
    default: return 'Enter your account number';
    }
  }

  inputWrapClass(user) {
    const classes = ['login-form__input-wrap'];

    if(user.status === LoginState.connecting) {
      classes.push('login-form__input-wrap--inactive');
    }

    return classes.join(' ');
  }

  inputClass(user) {
    const map = {
      [LoginState.failed]: 'login-form__input-field--error'
    };
    const classes = ['login-form__input-field'];
    const extra = map[user.status];

    return classes.concat(extra ? extra : []).join(' ');
  }

  footerClass(user) {
    const map = {
      [LoginState.ok]: 'login-footer--invisible',
      [LoginState.connecting]: 'login-footer--invisible'
    };
    const classes = ['login-footer'];
    const extra = map[user.status];

    return classes.concat(extra ? extra : []).join(' ');
  }

  submitClass(user) {
    const classes = ['login-form__submit'];

    if(user.account.length > 0) {
      classes.push('login-form__submit--active');
    }

    if(user.status === LoginState.connecting) {
      classes.push('login-form__submit--invisible');
    }

    return classes.join(' ');
  }

  render() {
    const { account, status, error } = this.props.user;
    const title = this.formTitle(status);
    const subtitle = this.formSubtitle(status, error);
    const displayAccount = formatAccount(account);
    const isConnecting = status === LoginState.connecting;
    const isFailed = status === LoginState.failed;
    const isLoggedIn = status === LoginState.ok;

    const inputWrapClass = this.inputWrapClass(this.props.user);
    const inputClass = this.inputClass(this.props.user);
    const footerClass = this.footerClass(this.props.user);
    const submitClass = this.submitClass(this.props.user);

    const autoFocusRef = input => {
      if(isFailed && input) {
        input.focus();
      }
    };

    return (
      <Layout>
        <Header />
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
                  <input className={ inputClass } 
                        type="text" 
                        placeholder="e.g 0000 0000 0000" 
                        onChange={ ::this.handleInputChange }
                        onKeyUp={ ::this.handleInputKeyUp }
                        value={ displayAccount }
                        disabled={ isConnecting }
                        autoFocus={ true } 
                        ref={ autoFocusRef } />
                    <button className={ submitClass } onClick={ ::this.handleLogin }></button>
                </div>
              </div>
            </div>
            <div className={footerClass}>
              <div className="login-footer__prompt">Don't have an account number?</div>
              <button className="login-footer__button" onClick={ ::this.handleCreateAccount }>Create account</button>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
