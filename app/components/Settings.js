import React, { Component, PropTypes } from 'react';
import { Layout, Container, Header } from './Layout';

export default class Settings extends Component {

  static propTypes = {
    logout: PropTypes.func.isRequired
  }

  onClose() {
    this.props.router.push('/connect');
  }

  render() {
    return (
      <Layout>
        <Header hidden={ true } style={ Header.Style.defaultDark } />
        <Container>
          <div className="settings">
            <button className="settings__close" onClick={ ::this.onClose } />
            <div className="settings__container">
              <div className="settings__header">
                <h2 className="settings__title">Settings</h2>
                <div className="settings__account">
                  <div className="settings__account-label">Account ID</div>
                  <div className="settings__account-id">{ this.props.user.account }</div>
                </div>
              </div>
              <div className="settings__content">
                <div className="settings__options">
                  <div className="settings__cell">
                    <div className="settings__cell-label">Auto-secure</div>
                    <div className="settings__cell-value">
                      <input type="checkbox" className="settings__switch" />
                    </div>
                  </div>
                  <div className="settings__cell-footer">
                    When this device connects to the internet it will automatically connect to a secure server
                  </div>
                  <div className="settings__cell">
                    <div className="settings__cell-label">FAQs</div>
                  </div>
                  <div className="settings__cell">
                    <div className="settings__cell-label">Guides</div>
                  </div>
                  <div className="settings__cell">
                    <img className="settings__cell-icon" src="./assets/images/icon-email.svg" />
                    <div className="settings__cell-label">Contact support</div>
                  </div>
                </div>
                <div className="settings__footer">
                  <button className="settings__logout-button">Logout</button>
                </div>
              </div>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
