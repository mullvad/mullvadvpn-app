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
    this.props.onLogin(this.props.backend, this.state.account);
  }

  handleCreateAccount() {
    shell.openExternal(constants.createAccountURL);
  }

  handleInputChange(e) {
    const val = e.target.value.replace(/[^0-9]/g, '');
    this.setState({ account: val });
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
                       value={this.formattedAccount(this.state.account)} />
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
