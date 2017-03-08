import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { If, Then, Else } from 'react-if';
import { Layout, Container, Header } from './Layout';
import Switch from './Switch';
import CustomScrollbars from './CustomScrollbars';
import { LoginState } from '../enums';

export default class Settings extends Component {

  static propTypes = {
    onQuit: PropTypes.func.isRequired,
    onLogout: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired,
    onViewAccount: PropTypes.func.isRequired,
    onExternalLink: PropTypes.func.isRequired,
    onUpdateSettings: PropTypes.func.isRequired
  }

  onClose() {
    this.props.onClose();
  }

  onAutoSecure(isOn) {
    this.props.onUpdateSettings({ autoSecure: isOn });
  }

  onExternalLink(type) {
    this.props.onExternalLink(type);
  }

  onLogout() {
    this.props.onLogout();
  }

  render() {
    const isLoggedIn = this.props.user.status === LoginState.ok;
    let isOutOfTime = false, formattedPaidUntil = '';
    let paidUntilIso = this.props.user.paidUntil;

    if(isLoggedIn && paidUntilIso) {
      let paidUntil = moment(this.props.user.paidUntil);
      isOutOfTime = paidUntil.isSameOrBefore(moment());
      formattedPaidUntil = paidUntil.fromNow(true) + ' left';
    }

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

                    { /* show account options when logged in */ }
                    <If condition={ isLoggedIn }>
                      <Then>
                        <div>

                          <div className="settings__cell settings__cell--active" onClick={ this.props.onViewAccount }>
                            <div className="settings__cell-label">Account</div>
                            <div className="settings__cell-value">
                              <If condition={ isOutOfTime }>
                                <Then>
                                  <span className="settings__account-paid-until-label settings__account-paid-until-label--error">OUT OF TIME</span>
                                </Then>
                                <Else>
                                  <span className="settings__account-paid-until-label">{ formattedPaidUntil }</span>
                                </Else>
                              </If>
                            </div>
                            <img className="settings__cell-disclosure" src="assets/images/icon-chevron.svg" />
                          </div>
                          <div className="settings__cell-spacer"></div>

                          <div className="settings__cell">
                            <div className="settings__cell-label">Auto-connect</div>
                            <div className="settings__cell-value">
                              <Switch onChange={ ::this.onAutoSecure } isOn={ this.props.settings.autoSecure } />
                            </div>
                          </div>
                          <div className="settings__cell-footer">
                            When this device connects to the internet it will automatically connect to a secure server
                          </div>
                        </div>
                      </Then>
                    </If>

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

                  <div className="settings__footer">
                    <button className="button button--negative" onClick={ this.props.onQuit }>Quit app</button>
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
