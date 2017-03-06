import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { formatAccount } from '../lib/formatters';
import { LoginState } from '../enums';

export default class Account extends Component {

  static propTypes = {
    onLogout: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired,
    onExternalLink: PropTypes.func.isRequired
  }

  onClose() {
    this.props.onClose();
  }

  onExternalLink(type) {
    this.props.onExternalLink(type);
  }

  onLogout() {
    this.props.onLogout();
  }

  render() {
    const isLoggedIn = this.props.user.status === LoginState.ok;
    let formattedAccountId = formatAccount(this.props.user.account);

    return (
      <Layout>
        <Header hidden={ true } style={ Header.Style.defaultDark } />
        <Container>
          <div className="account">
            <div className="account__close" onClick={ ::this.onClose }>
              <img className="account__close-icon" src="./assets/images/icon-back.svg" />
              <span className="account__close-title">Settings</span>
            </div>
            <div className="account__container">
            
              <div className="account__header">
                <h2 className="account__title">Account</h2>
              </div> 

              <div className="account__content">
                <div className="account__main">
                
                  <div className="account__row">
                    <div className="account__row-label">Account ID</div>
                    <div className="account__row-value">{ formattedAccountId }</div>
                  </div>

                  <div className="account__row">
                    <div className="account__row-label">Paid until</div>
                    <div className="account__row-value">4:20pm, 12 November 2017</div>
                  </div>

                  <div className="account__footer">
                    <button className="button button--positive" onClick={ this.onExternalLink.bind(this, 'purchase') }>Buy more time</button>
                    <button className="button button--negative" onClick={ ::this.onLogout }>Logout</button>
                  </div>

                </div>
              </div>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
