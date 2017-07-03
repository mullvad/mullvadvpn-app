// @flow
import moment from 'moment';
import React, { Component } from 'react';
import { If, Then, Else } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { formatAccount } from '../lib/formatters';
import ExternalLinkSVG from '../assets/images/icon-extLink.svg';

import type { UserReduxState } from '../reducers/user';

export type AccountProps = {
  user: UserReduxState;
  onLogout: () => void;
  onClose: () => void;
  onExternalLink: (type: string) => void;
};

export default class Account extends Component {
  props: AccountProps;

  onBuyMore = () => this.props.onExternalLink('purchase');
  onClose = () => this.props.onClose();
  onLogout = () => this.props.onLogout();

  render(): React.Element<*> {
    const user = this.props.user;
    const paidUntil = moment(user.paidUntil);
    const formattedAccountId = formatAccount(user.account || '');
    const formattedPaidUntil = paidUntil.format('hA, D MMMM YYYY').toUpperCase();
    const isOutOfTime = paidUntil.isSameOrBefore(moment());

    return (
      <Layout>
        <Header hidden={ true } style={ 'defaultDark' } />
        <Container>
          <div className="account">
            <div className="account__close" onClick={ this.onClose }>
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
                    <If condition={ isOutOfTime }>
                      <Then>
                        <div className="account__row-value account__row-value--error">OUT OF TIME</div>
                      </Then>
                      <Else>
                        <div className="account__row-value">{ formattedPaidUntil }</div>
                      </Else>
                    </If>
                  </div>

                  <div className="account__footer">
                    <button className="button button--positive" onClick={ this.onBuyMore }>
                      <span className="button-label">Buy more time</span>
                      <ExternalLinkSVG className="button-icon button-icon--16" />
                    </button>
                    <button className="button button--negative" onClick={ this.onLogout }>Logout</button>
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
