import { shell } from 'electron';
import React, { Component, PropTypes } from 'react';
import { If, Then, Else } from 'react-if';
import Layout from './Layout';
import { createAccountURL, LoginState } from '../constants';

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
    shell.openExternal(createAccountURL);
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

  formattedAccount(val) {
    // display number altogether when longer than 12
    if(val.length > 12) {
      return val;
    }

    // display quartets
    return val.replace(/([0-9]{4})/g, '$1 ').trim();
  }

  formTitle(s) {
    switch(s) {
      case LoginState.connecting: return "Logging in...";
      case LoginState.failed: return "Login failed";
      case LoginState.ok: return "Logged in";
      default: return "Login";
    }
  }

  formSubtitle(s, e) {
    switch(s) {
      case LoginState.failed: return e.message;
      case LoginState.connecting: return 'Checking account number';
      default: return 'Enter your account number';
    }
  }

  render() {
    const { account, status, error } = this.props.user;
    const title = this.formTitle(status);
    const subtitle = this.formSubtitle(status, error);
    const displayAccount = this.formattedAccount(account);
    const isConnecting = status === LoginState.connecting;
    const isFailed = status === LoginState.failed;
    const inputClass = ["login-form__input-field", isFailed ? "login-form__input-field--error" : ""].join(' ');
    const footerClass = ["login-footer", isConnecting ? "login-footer--invisible" : ""].join(' ');
    
    const autoFocusRef = input => {
      if(isFailed && input) {
        input.focus();
      }
    };

    return (
      <Layout>
        <div className="login">
          <div className="login-form">
            <div>

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

              <div className="login-form__title">{ title }</div>
              <div className="login-form__subtitle">{ subtitle }</div>
              <div className="login-form__input-wrap">
                <input className={ inputClass } 
                       type="text" 
                       placeholder="0000 0000 0000" 
                       onChange={ ::this.handleInputChange }
                       onKeyUp={ ::this.handleInputKeyUp }
                       value={ displayAccount }
                       disabled={ isConnecting }
                       autoFocus={ true } 
                       ref={ autoFocusRef } />
              </div>
            </div>
          </div>
          <div className={footerClass}>
            <div className="login-footer__prompt">Don't have an account number?</div>
            <button className="login-footer__button" onClick={ ::this.handleCreateAccount }>Create account</button>
          </div>
        </div>
      </Layout>
    );
  }
}
