import React, { Component, PropTypes } from 'react';
import { Layout, Container, Header } from './Layout';
import Switch from './Switch';
import CustomScrollbars from './CustomScrollbars';
import { formatAccount } from '../lib/formatters';

export default class Settings extends Component {

  static propTypes = {
    logout: PropTypes.func.isRequired,
    openExternalLink: PropTypes.func.isRequired,
    updateSettings: PropTypes.func.isRequired
  }

  onClose() {
    this.props.router.push('/connect');
  }

  onAutoSecure(isOn) {
    this.props.updateSettings({ autoSecure: isOn });
  }

  onExternalLink(type) {
    this.props.openExternalLink(type);
  }

  onLogout() {
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
                        <Switch onChange={ ::this.onAutoSecure } isOn={ this.props.settings.autoSecure } />
                      </div>
                    </div>
                    <div className="settings__cell-footer">
                      When this device connects to the internet it will automatically connect to a secure server
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ this.onExternalLink.bind(this, 'faq') }>
                      <div className="settings__cell-label">FAQs</div>
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ this.onExternalLink.bind(this, 'guides') }>
                      <div className="settings__cell-label">Guides</div>
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ this.onExternalLink.bind(this, 'supportEmail') }>
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
                    <button className="button button--neutral" onClick={ this.onExternalLink.bind(this, 'purchase') }>Buy more time</button>
                    <button className="button button--negative" onClick={ ::this.onLogout }>Logout</button>
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
