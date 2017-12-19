// @flow
import moment from 'moment';
import React, { Component } from 'react';
import { If, Then, Else } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { formatAccount } from '../lib/formatters';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';

import type { AccountReduxState } from '../redux/account/reducers';

export type AccountProps = {
  account: AccountReduxState;
  onLogout: () => void;
  onClose: () => void;
  onBuyMore: () => void;
};

export default class Account extends Component {
  props: AccountProps;

  render(): React.Element<*> {
    const expiry = moment(this.props.account.expiry);
    const formattedAccountToken = formatAccount(this.props.account.accountToken || '');
    const formattedExpiry = expiry.format('hA, D MMMM YYYY').toUpperCase();
    const isOutOfTime = expiry.isSameOrBefore(moment());

    return (
      <Layout>
        <Header hidden={ true } style={ 'defaultDark' } />
        <Container>
          <div className="account">
            <div className="account__close" onClick={ this.props.onClose }>
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
                    <div className="account__row-value">{ formattedAccountToken }</div>
                  </div>

                  <div className="account__row">
                    <div className="account__row-label">Paid until</div>
                    <If condition={ isOutOfTime }>
                      <Then>
                        <div className="account__out-of-time account__row-value account__row-value--error">OUT OF TIME</div>
                      </Then>
                      <Else>
                        <div className="account__row-value">{ formattedExpiry }</div>
                      </Else>
                    </If>
                  </div>

                  <div className="account__footer">
                    <button className="account__buymore button button--positive" onClick={ this.props.onBuyMore }>
                      <span className="button-label">Buy more time</span>
                      <ExternalLinkSVG className="button-icon button-icon--16" />
                    </button>
                    <button className="account__logout button button--negative" onClick={ this.props.onLogout }>Logout</button>
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
