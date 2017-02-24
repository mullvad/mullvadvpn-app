import React, { Component, PropTypes } from 'react';
import { Layout, Container, Header } from './Layout';
import Switch from './Switch';
import CustomScrollbars from './CustomScrollbars';
import { formatAccount } from '../lib/formatters';
import { links } from '../constants';
import { shell } from 'electron';

export default class Settings extends Component {

  static propTypes = {
    logout: PropTypes.func.isRequired,
    buyTime: PropTypes.func.isRequired,
    updateSettings: PropTypes.func.isRequired
  }

  onClose() {
    this.props.router.push('/connect');
  }

  handleAutoSecure(isOn) {
    this.props.updateSettings({ autoSecure: isOn });
  }

  handleLink(key) {
    shell.openExternal(links[key]);
  }

  handleBuy() {
    this.props.buyTime();
  }

  handleLogout() {
    this.props.logout();
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
              </div>
              <CustomScrollbars autoHide={ true }>
                <div className="settings__content">
                  <div className="settings__main">
                    <div className="settings__cell">
                      <div className="settings__cell-label">Auto-secure</div>
                      <div className="settings__cell-value">
                        <Switch onChange={ ::this.handleAutoSecure } isOn={ this.props.settings.autoSecure } />
                      </div>
                    </div>
                    <div className="settings__cell-footer">
                      When this device connects to the internet it will automatically connect to a secure server
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ () => this.handleLink('faq') }>
                      <div className="settings__cell-label">FAQs</div>
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ () => this.handleLink('guides') }>
                      <div className="settings__cell-label">Guides</div>
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ () => this.handleLink('supportEmail') }>
                      <img className="settings__cell-icon" src="./assets/images/icon-email.svg" />
                      <div className="settings__cell-label">Contact support</div>
                    </div>
                  </div>
                  <div className="settings__account">
                    <div className="settings__account-row">
                      <div className="settings__account-label">Account ID</div>
                      <div className="settings__account-id">{ formatAccount(this.props.user.account) }</div>
                    </div>
                    <div className="settings__account-row">
                      <div className="settings__account-label">Time remaining</div>
                      <div className="settings__account-id">12 days</div>
                    </div>
                  </div>
                  <div className="settings__footer">
                    <button className="button button--neutral" onClick={ ::this.handleBuy }>Buy more time</button>
                    <button className="button button--negative" onClick={ ::this.handleLogout }>Logout</button>
                  </div>
                </div>
              </CustomScrollbars>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
