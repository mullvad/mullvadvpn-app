// @flow
import moment from 'moment';
import React from 'react';
import { Component, Text, View } from 'reactxp';
import { Button } from './styled';
import { Layout, Container, Header } from './Layout';
import CustomScrollbars from './CustomScrollbars';
import styles from './SettingsStyles';
import Img from './Img';

import type { AccountReduxState } from '../redux/account/reducers';
import type { SettingsReduxState } from '../redux/settings/reducers';

export type SettingsProps = {
  account: AccountReduxState,
  settings: SettingsReduxState,
  onQuit: () => void,
  onClose: () => void,
  onViewAccount: () => void,
  onViewSupport: () => void,
  onViewPreferences: () => void,
  onViewAdvancedSettings: () => void,
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
          <View style={styles.settings}>
            <Button style={styles.settings__close} onPress={ this.props.onClose } testName='settings__close'>
              <Img style={styles.settings__close_icon} source='icon-close'/>
            </Button>

            <View style={styles.settings__container}>
              <View style={styles.settings__header}>
                <Text style={styles.settings__title}>Settings</Text>
              </View>

              <CustomScrollbars autoHide={ true }>

                <View style={styles.settings__content}>
                  <View style={styles.settings__main}>
                    {/* show account options when logged in */}
                    {isLoggedIn ? (
                      <View style={styles.settings_account} testName='settings__account'>

                        <Button onPress={ this.props.onViewAccount } testName='settings__view_account'>
                          <View style={styles.settings__cell}>
                            <Text style={styles.settings__cell_label}>Account</Text>
                            <View style={styles.settings__account_paid_until_label_container}>
                              {isOutOfTime ? (
                                <Text style={styles.settings__account_paid_until_label__error} testName='settings__account_paid_until_label'>OUT OF TIME</Text>
                              ) : (
                                <Text style={styles.settings__account_paid_until_label} testName='settings__account_paid_until_label'>{formattedExpiry}</Text>
                              )}
                            </View>
                            <Img style={styles.settings__cell_disclosure} source='icon-chevron' tintColor='currentColor'/>
                          </View>
                        </Button>
                      </View>
                    ) : null}

                    {isLoggedIn ? (
                      <Button onPress={ this.props.onViewPreferences } testName='settings__preferences'>
                        <View style={styles.settings__cell}>
                          <Text style={styles.settings__cell_label}>Preferences</Text>
                          <Img style={styles.settings__cell_disclosure} source='icon-chevron' tintColor='currentColor' />
                        </View>
                      </Button>
                    ) : null}

                    {isLoggedIn ? (
                      <Button onPress={ this.props.onViewAdvancedSettings } testName="settings__advanced">
                        <View style={styles.settings__cell}>
                          <Text style={styles.settings__cell_label}>Advanced</Text>
                          <Img style={styles.settings__cell_disclosure} source='icon-chevron' tintColor='currentColor'/>
                        </View>
                      </Button>
                    ) : null}

                    {isLoggedIn ? (
                      <View style={styles.settings__cell_spacer}></View>
                    ) : null}

                    <Button onPress={ this.props.onExternalLink.bind(this, 'faq') } testName='settings__external_link'>
                      <View style={styles.settings__cell}>
                        <Text style={styles.settings__cell_label}>FAQs</Text>
                        <Img style={styles.settings__cell_icon} source='icon-extLink' tintColor='currentColor'/>
                      </View>
                    </Button>

                    <Button onPress={ this.props.onExternalLink.bind(this, 'guides') } testName='settings__external_link'>
                      <View style={styles.settings__cell}>
                        <Text style={styles.settings__cell_label}>Guides</Text>
                        <Img style={styles.settings__cell_icon} source='icon-extLink' tintColor='currentColor'/>
                      </View>
                    </Button>

                    <Button onPress={ this.props.onViewSupport }  testName='settings__view_support'>
                      <View style={styles.settings__cell}>
                        <Text style={styles.settings__cell_label}>Contact support</Text>
                        <Img style={styles.settings__cell_disclosure} source='icon-chevron' tintColor='currentColor'/>
                      </View>
                    </Button>
                  </View>

                  <View style={styles.settings__footer}>
                    <Button style={styles.settings__footer_button} onPress={this.props.onQuit} testName='settings__quit'>
                      <Text style={styles.settings__footer_button_label}>Quit app</Text>
                    </Button>
                  </View>
                </View>
              </CustomScrollbars>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }
}
