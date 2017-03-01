import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { formatAccount } from '../lib/formatters';
import { LoginState } from '../enums';

export default class Login extends Component {
  static propTypes = {
    user: PropTypes.object.isRequired,
    onLogin: PropTypes.func.isRequired,
    onChange: PropTypes.func.isRequired,
    onFirstChangeAfterFailure: PropTypes.func.isRequired,
    onExternalLink: PropTypes.func.isRequired,
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

  onLogin() {
    const { account } = this.props.user;
    if(account.length > 0) {
      this.props.onLogin(account);
    }
  }

  onCreateAccount() {
    this.props.onExternalLink('createAccount');
  }

  onInputChange(e) {
    const val = e.target.value.replace(/[^0-9]/g, '');

    // notify delegate on first change after login failure
    if(this.state.notifyOnFirstChangeAfterFailure) {
      this.setState({ notifyOnFirstChangeAfterFailure: false });
      this.props.onFirstChangeAfterFailure();
    }

    this.props.onChange(val);
  }

  onInputKeyUp(e) {
    if(e.which === 13) {
      this.onLogin();
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
    switch(user.status) {
    case LoginState.connecting:
      classes.push('login-form__input-wrap--inactive');
      break;
    case LoginState.failed:
      classes.push('login-form__input-wrap--error');
      break;
    }

    return classes.join(' ');
  }

  footerClass(user) {
    const classes = ['login-footer'];
    switch(user.status) {
    case LoginState.ok:
    case LoginState.connecting:
      classes.push('login-footer--invisible');
      break;
    }
    return classes.join(' ');
  }

  submitClass(user) {
    const classes = ['login-form__submit'];

    if(typeof(user.account) === 'string' && user.account.length > 0) {
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
    const displayAccount = formatAccount(account || '');
    const isConnecting = status === LoginState.connecting;
    const isFailed = status === LoginState.failed;
    const isLoggedIn = status === LoginState.ok;

    const inputWrapClass = this.inputWrapClass(this.props.user);
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
                  <input className="login-form__input-field" 
                        type="text" 
                        placeholder="e.g 0000 0000 0000" 
                        onChange={ ::this.onInputChange }
                        onKeyUp={ ::this.onInputKeyUp }
                        value={ displayAccount }
                        disabled={ isConnecting }
                        autoFocus={ true } 
                        ref={ autoFocusRef } />
                    <button className={ submitClass } onClick={ ::this.onLogin }></button>
                </div>
              </div>
            </div>
            <div className={footerClass}>
              <div className="login-footer__prompt">Don't have an account number?</div>
              <button className="button button--primary" onClick={ ::this.onCreateAccount }>Create account</button>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
