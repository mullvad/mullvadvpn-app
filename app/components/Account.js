// @flow
import moment from 'moment';
import React from 'react';
import { Component, Text, View } from 'reactxp';
import { Button, AppButton } from './styled';
import { Layout, Container } from './Layout';
import styles from './AccountStyles';
import Img from './Img';
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

  render() {
    const expiry = moment(this.props.account.expiry);
    const formattedAccountToken = formatAccount(this.props.account.accountToken || '');
    const formattedExpiry = expiry.format('hA, D MMMM YYYY').toUpperCase();
    const isOutOfTime = expiry.isSameOrBefore(moment());

    return (
      <Layout>
        <Container>
          <View style={styles.account}>
            <Button style={styles.account__close} onPress={ this.props.onClose }>
              <Img style={styles.account__close_icon} source="icon-back" />
              <Text style={styles.account__close_title}>Settings</Text>
            </Button>
            <View style={styles.account__container}>

              <View style={styles.account__header}>
                <Text style={styles.account__title}>Account</Text>
              </View>

              <View style={styles.account__content}>
                <View style={styles.account__main}>

                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Account ID</Text>
                    <Text style={styles.account__row_value}>{ formattedAccountToken }</Text>
                  </View>

                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Paid until</Text>
                    { isOutOfTime ?
                      <Text style={styles.account__out_of_time}>OUT OF TIME</Text>
                      :
                      <Text style={styles.account__row_value}>{ formattedExpiry }</Text>
                    }
                  </View>

                  <View style={styles.account__footer}>
                    <AppButton style={styles.account__buymore}
                      hoverStyle={styles.account__buymore_hover}
                      onPress={ this.props.onBuyMore }
                      text='Buy more credit'
                      icon='icon-extLink'
                      iconStyle={styles.account__buymore_icon}
                      tintColor='currentColor'/>
                    <AppButton style={styles.account__logout}
                      hoverStyle={styles.account__logout_hover}
                      onPress={ this.props.onLogout }
                      text='Log out'/>
                  </View>

                </View>
              </View>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }
}
