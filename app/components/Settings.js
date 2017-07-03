// @flow
import moment from 'moment';
import React, { Component } from 'react';
import { If, Then, Else } from 'react-if';
import { Layout, Container, Header } from './Layout';
import Switch from './Switch';
import CustomScrollbars from './CustomScrollbars';

import type { AccountReduxState } from '../redux/account/reducers';
import type { SettingsReduxState } from '../redux/settings/reducers';

export type SettingsProps = {
  account: AccountReduxState,
  settings: SettingsReduxState,
  onQuit: () => void,
  onClose: () => void,
  onViewAccount: () => void,
  onExternalLink: (type: string) => void,
  onUpdateSettings: (update: $Shape<SettingsReduxState>) => void
};

export default class Settings extends Component {

  props: SettingsProps;

  onClose = () => this.props.onClose();
  onAutoSecure = (autoSecure: boolean) => this.props.onUpdateSettings({ autoSecure });

  onExternalLink(type: string) {
    this.props.onExternalLink(type);
  }

  render(): React.Element<*> {
    const isLoggedIn = this.props.account.status === 'ok';
    let isOutOfTime = false, formattedPaidUntil = '';
    let paidUntilIso = this.props.account.paidUntil;

    if(isLoggedIn && paidUntilIso) {
      let paidUntil = moment(this.props.account.paidUntil);
      isOutOfTime = paidUntil.isSameOrBefore(moment());
      formattedPaidUntil = paidUntil.fromNow(true) + ' left';
    }

    return (
      <Layout>
        <Header hidden={ true } style={ 'defaultDark' } />
        <Container>
          <div className="settings">
            <button className="settings__close" onClick={ this.onClose } />
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
                              <Switch onChange={ this.onAutoSecure } isOn={ this.props.settings.autoSecure } />
                            </div>
                          </div>
                          <div className="settings__cell-footer">
                            When this device connects to the internet, Mullvad VPN will automatically secure your connection
                          </div>
                        </div>
                      </Then>
                    </If>

                    <div className="settings__cell settings__cell--active" onClick={ this.onExternalLink.bind(this, 'faq') }>
                      <div className="settings__cell-label">FAQs</div>
                      <img className="settings__cell-icon" src="./assets/images/icon-extLink.svg" />
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ this.onExternalLink.bind(this, 'guides') }>
                      <div className="settings__cell-label">Guides</div>
                      <img className="settings__cell-icon" src="./assets/images/icon-extLink.svg" />
                    </div>
                    <div className="settings__cell settings__cell--active" onClick={ this.onExternalLink.bind(this, 'supportEmail') }>
                      <div className="settings__cell-label">Contact support</div>
                      <img className="settings__cell-icon" src="./assets/images/icon-email.svg" />
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
