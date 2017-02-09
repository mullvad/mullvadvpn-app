import React, { Component, PropTypes } from 'react';
import Layout from './Layout';

export default class Login extends Component {
  static propTypes = {
    onLogin: PropTypes.func.isRequired
  };

  handleLogin() {
    const { onLogin } = this.props;
    const username = this.refs.username.value;

    onLogin({ username, loggedIn: true });

    this.props.router.push('/loggedin');
  }

  render() {
    return (
      <Layout>
        <div className="login">
          <div className="login-form">
            <div className="login-form__alignbox">
              <div className="login-form__title">Login</div>
              <div className="login-form__subtitle">Enter your account number</div>
              <div className="login-form__input-wrap">
                <input className="login-form__input-field" type="text" placeholder="0000 0000 0000" />
              </div>
            </div>
          </div>
          <div className="login-footer">
            <div className="login-footer__prompt">Don't have an account number?</div>
            <button className="login-footer__button">Create account</button>
          </div>
        </div>
      </Layout>
    );
  }
}
