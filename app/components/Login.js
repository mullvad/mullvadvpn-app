import { shell } from 'electron';
import React, { Component, PropTypes } from 'react';
import Layout from './Layout';
import constants from '../constants';

export default class Login extends Component {
  static propTypes = {
    onLogin: PropTypes.func.isRequired
  };

  constructor(props) {
    super(props);

    this.state = { account: '' };
  }

  handleLogin() {
    const { onLogin } = this.props;
    const username = this.refs.username.value;

    onLogin({ username, loggedIn: true });
  }

  handleCreateAccount() {
    shell.openExternal(constants.createAccountURL);
  }

  handleInputChange(e) {
    let val = e.target.value.replace(/[^0-9]/g, '');
    this.setState({ account: val });
  }

  handleInputKeyUp(e) {
    if(e.which === 13) {
      // enter pressed
    }
  }

  render() {
    return (
      <Layout>
        <div className="login">
          <div className="login-form">
            <div>
              <div className="login-form__title">Login</div>
              <div className="login-form__subtitle">Enter your account number</div>
              <div className="login-form__input-wrap">
                <input className="login-form__input-field" 
                       type="text" 
                       placeholder="0000 0000 0000" 
                       onChange={::this.handleInputChange}
                       onKeyUp={::this.handleInputKeyUp}
                       value={this.state.account} />
              </div>
            </div>
          </div>
          <div className="login-footer">
            <div className="login-footer__prompt">Don't have an account number?</div>
            <button className="login-footer__button" onClick={::this.handleCreateAccount}>Create account</button>
          </div>
        </div>
      </Layout>
    );
  }
}
