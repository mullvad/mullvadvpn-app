import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import AccountInput from './AccountInput';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';
import LoginArrowSVG from '../assets/images/icon-arrow.svg';

export default class Login extends Component {
  static propTypes = {
    user: PropTypes.object.isRequired,
    onLogin: PropTypes.func.isRequired,
    onSettings: PropTypes.func.isRequired,
    onChange: PropTypes.func.isRequired,
    onFirstChangeAfterFailure: PropTypes.func.isRequired,
    onExternalLink: PropTypes.func.isRequired,
  };

  constructor(props) {
    super(props);
    this.state = {
      notifyOnFirstChangeAfterFailure: false,
      isActive: false
    };
  }

  componentWillReceiveProps(nextProps) {
    const prev = this.props.user || {};
    const next = nextProps.user || {};

    if(prev.status !== next.status && next.status === 'failed') {
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

  onFocus() {
    this.setState({ isActive: true });
  }

  onBlur() {
    this.setState({ isActive: false });
  }

  onInputChange(val) {
    // notify delegate on first change after login failure
    if(this.state.notifyOnFirstChangeAfterFailure) {
      this.setState({ notifyOnFirstChangeAfterFailure: false });
      this.props.onFirstChangeAfterFailure();
    }
    this.props.onChange(val);
  }

  formTitle(s) {
    switch(s) {
    case 'connecting': return 'Logging in...';
    case 'failed': return 'Login failed';
    case 'ok': return 'Login successful';
    default: return 'Login';
    }
  }

  formSubtitle(s, e) {
    switch(s) {
    case 'failed': return e.message;
    case 'connecting': return 'Checking account number';
    default: return 'Enter your account number';
    }
  }

  inputWrapClass(user) {
    const classes = ['login-form__input-wrap'];

    if(this.state.isActive) {
      classes.push('login-form__input-wrap--active');
    }

    switch(user.status) {
    case 'connecting':
      classes.push('login-form__input-wrap--inactive');
      break;
    case 'failed':
      classes.push('login-form__input-wrap--error');
      break;
    }

    return classes.join(' ');
  }

  footerClass(user) {
    const classes = ['login-footer'];
    switch(user.status) {
    case 'ok':
    case 'connecting':
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

    if(user.status === 'connecting') {
      classes.push('login-form__submit--invisible');
    }

    return classes.join(' ');
  }

  render() {
    const { account, status, error } = this.props.user;
    const title = this.formTitle(status);
    const subtitle = this.formSubtitle(status, error);
    const isConnecting = status === 'connecting';
    const isFailed = status === 'failed';
    const isLoggedIn = status === 'ok';

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
                        onFocus={ ::this.onFocus }
                        onBlur={ ::this.onBlur }
                        onChange={ ::this.onInputChange }
                        onEnter={ ::this.onLogin }
                        value={ account }
                        disabled={ isConnecting }
                        autoFocus={ true }
                        ref={ autoFocusRef } />
                    <button className={ submitClass } onClick={ ::this.onLogin }>
                      <LoginArrowSVG className="login-form__submit-icon" />
                    </button>
                </div>
              </div>
            </div>
            <div className={ footerClass }>
              <div className="login-footer__prompt">Don't have an account number?</div>
              <button className="button button--primary" onClick={ ::this.onCreateAccount }>
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
