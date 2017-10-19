// @flow
import moment from 'moment';
import React, { Component } from 'react';
import { If, Then, Else } from 'react-if';
import { Layout, Container, Header } from './Layout';
import CustomScrollbars from './CustomScrollbars';

import type { AccountReduxState } from '../redux/account/reducers';
import type { SettingsReduxState } from '../redux/settings/reducers';

export type SettingsProps = {
  account: AccountReduxState,
  settings: SettingsReduxState,
  onQuit: () => void,
  onClose: () => void,
  onViewAccount: () => void,
  onViewSupport: () => void,
  onExternalLink: (type: string) => void
};

export default class Settings extends Component {

  props: SettingsProps;

  render() {
    const isLoggedIn = this.props.account.status === 'ok';
    let isOutOfTime = false, formattedExpiry = '';
    let expiryIso = this.props.account.expiry;

    if(isLoggedIn && expiryIso) {
      let expiry = moment(this.props.account.expiry);
      isOutOfTime = expiry.isSameOrBefore(moment());
      formattedExpiry = expiry.fromNow(true) + ' left';
    }

    return (
      <Layout>
        <Header hidden={ true } style={ 'defaultDark' } />
        <Container>
          <div className="settings">
            <button className="settings__close" onClick={ this.props.onClose } />
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
                        <div className="settings__account">

                          <div className="settings__view-account settings__cell settings__cell--active" onClick={ this.props.onViewAccount }>
                            <div className="settings__cell-label">Account</div>
                            <div className="settings__cell-value">
                              <If condition={ isOutOfTime }>
                                <Then>
                                  <span className="settings__account-paid-until-label settings__account-paid-until-label--error">OUT OF TIME</span>
                                </Then>
                                <Else>
                                  <span className="settings__account-paid-until-label">{ formattedExpiry }</span>
                                </Else>
                              </If>
                            </div>
                            <img className="settings__cell-disclosure" src="assets/images/icon-chevron.svg" />
                          </div>
                          <div className="settings__cell-spacer"></div>

                          <div className="settings__cell-footer"></div>
                        </div>
                      </Then>
                    </If>

                    <div className="settings__external">
                      <div className="settings__cell settings__cell--active" onClick={ this.props.onExternalLink.bind(this, 'faq') }>
                        <div className="settings__cell-label">FAQs</div>
                        <img className="settings__cell-icon" src="./assets/images/icon-extLink.svg" />
                      </div>
                      <div className="settings__cell settings__cell--active" onClick={ this.props.onExternalLink.bind(this, 'guides') }>
                        <div className="settings__cell-label">Guides</div>
                        <img className="settings__cell-icon" src="./assets/images/icon-extLink.svg" />
                      </div>
                      <div className="settings__view-support settings__cell settings__cell--active" onClick={ this.props.onViewSupport }>
                        <div className="settings__cell-label">Contact support</div>
                        <img className="settings__cell-disclosure" src="assets/images/icon-chevron.svg" />
                      </div>
                    </div>
                  </div>

                  <div className="settings__footer">
                    <button className="settings__quit button button--negative" onClick={ this.props.onQuit }>Quit app</button>
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
