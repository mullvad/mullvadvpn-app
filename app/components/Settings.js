// @flow
import moment from 'moment';
import React from 'react';
import { Component, Text, View } from 'reactxp';
import { Button, CellButton } from './styled';
import { Layout, Container, Header } from './Layout';
import CustomScrollbars from './CustomScrollbars';
import styles from './SettingsStyles';
import Img from './Img';

import type { AccountReduxState } from '../redux/account/reducers';
import type { SettingsReduxState } from '../redux/settings/reducers';

export type SettingsProps = {
  account: AccountReduxState,
  settings: SettingsReduxState,
  version: string,
  onQuit: () => void,
  onClose: () => void,
  onViewAccount: () => void,
  onViewSupport: () => void,
  onViewPreferences: () => void,
  onViewAdvancedSettings: () => void,
  onExternalLink: (type: string) => void,
};

export default class Settings extends Component {

  props: SettingsProps;

  render() {
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

              <CustomScrollbars style={styles.settings__scrollview} autoHide={ true }>

                <View style={styles.settings__content}>
                  <View>
                    { this._renderTopButtons() }
                    <View style={styles.settings__cell_spacer}/>
                    { this._renderMiddleButtons() }
                    <View style={styles.settings__cell_spacer}/>
                    { this._renderBottomButtons() }
                  </View>
                  { this._renderQuitButton() }
                </View>

              </CustomScrollbars>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }

  _renderTopButtons() {
    const isLoggedIn = this.props.account.status === 'ok';
    if (!isLoggedIn) {
      return null;
    }

    let isOutOfTime = false, formattedExpiry = '';
    let expiryIso = this.props.account.expiry;

    if(isLoggedIn && expiryIso) {
      let expiry = moment(this.props.account.expiry);
      isOutOfTime = expiry.isSameOrBefore(moment());
      formattedExpiry = (expiry.fromNow(true) + ' left').toUpperCase();
    }

    return <View>
      <View style={styles.settings_account} testName='settings__account'>
        {!isOutOfTime ? (
          <CellButton onPress={ this.props.onViewAccount }
            text='Account'
            testName='settings__account_paid_until_label'
            subtext='OUT OF TIME'
            subtextStyle={ styles.settings__account_paid_until_label__error }
            icon='icon-chevron'
            iconStyle={styles.settings__icon_chevron}
            tintColor='currentColor'/>
        ) : (
          <CellButton onPress={ this.props.onViewAccount }
            text='Account'
            testName='settings__account_paid_until_label'
            subtext={ formattedExpiry }
            icon='icon-chevron'
            iconStyle={styles.settings__icon_chevron}
            tintColor='currentColor'/>
        )}
      </View>

      <CellButton onPress={ this.props.onViewPreferences }
        testName='settings__preferences'
        text='Preferences'
        icon='icon-chevron'
        tintColor='currentColor'/>

      <CellButton onPress={ this.props.onViewAdvancedSettings }
        testName='settings__advanced'
        text='Advanced'
        icon='icon-chevron'
        tintColor='currentColor'/>
    </View>;
  }

  _renderMiddleButtons() {
    return <CellButton onPress={ this.props.onExternalLink.bind(this, 'download') }
      testName='settings__version'
      text='App version'
      subtext={this._formattedVersion()}
      icon='icon-extLink'
      tintColor='currentColor'/>;
  }

  _formattedVersion() {
    // the version in package.json has to be semver, but we use a YEAR.release-channel
    // version scheme. in package.json we thus have to write YEAR.release.X-channel and
    // this function is responsible for removing .X part.
    return this.props.version
      .replace('.0-', '-')  // remove the .0 in 2018.1.0-beta9
      .replace(/\.0$/, ''); // remove the .0 in 2018.1.0
  }

  _renderBottomButtons() {
    return <View>
      <CellButton onPress={ this.props.onExternalLink.bind(this, 'faq') }
        testName='settings__external_link'
        text='FAQs'
        icon='icon-extLink'
        tintColor='currentColor'/>

      <CellButton onPress={ this.props.onExternalLink.bind(this, 'guides') }
        testName='settings__external_link'
        text='Guides'
        icon='icon-extLink'
        tintColor='currentColor'/>

      <CellButton onPress={ this.props.onViewSupport }
        testName='settings__view_support'
        text='Contact support'
        icon='icon-chevron'
        iconStyle={styles.settings__icon_chevron}
        tintColor='currentColor'/>
    </View>;
  }

  _renderQuitButton() {
    return <View style={styles.settings__footer}>
      <Button style={styles.settings__footer_button} onPress={this.props.onQuit} testName='settings__quit'>
        <Text style={styles.settings__footer_button_label}>Quit app</Text>
      </Button>
    </View>;
  }
}