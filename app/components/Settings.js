// @flow
import moment from 'moment';
import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { Button, CellButton, RedButton, Label, SubText} from './styled';
import { Layout, Container } from './Layout';
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

export default class Settings extends Component<SettingsProps> {
  render() {
    return (
      <Layout>
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
                    { this._renderMiddleButtons() }
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
        {isOutOfTime ? (
          <CellButton onPress={ this.props.onViewAccount }
            testName='settings__account_paid_until_button'>
            <Label>Account</Label>
            <SubText testName='settings__account_paid_until_subtext' style={styles.settings__account_paid_until_Label__error}>OUT OF TIME</SubText>
            <Img height={12} width={7} source='icon-chevron' />
          </CellButton>
        ) : (
          <CellButton onPress={ this.props.onViewAccount }
            testName='settings__account_paid_until_button'>
            <Label>Account</Label>
            <SubText testName='settings__account_paid_until_subtext'>{ formattedExpiry }</SubText>
            <Img height={12} width={7} source='icon-chevron' />
          </CellButton>
        )}
      </View>

      <CellButton onPress={ this.props.onViewPreferences }
        testName='settings__preferences'>
        <Label>Preferences</Label>
        <Img height={12} width={7} source='icon-chevron' />
      </CellButton>

      <CellButton onPress={ this.props.onViewAdvancedSettings }
        testName='settings__advanced'>
        <Label>Advanced</Label>
        <Img height={12} width={7} source='icon-chevron' />
      </CellButton>
      <View style={styles.settings__cell_spacer}/>
    </View>;
  }

  _renderMiddleButtons() {
    return <View>
      <CellButton onPress={ this.props.onExternalLink.bind(this, 'download') }
        testName='settings__version'>
        <Label>App version</Label>
        <SubText>{this._formattedVersion()}</SubText>
        <Img height={16} width={16} source='icon-extLink' />
      </CellButton>
      <View style={styles.settings__cell_spacer}/>
    </View>;
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
        testName='settings__external_link'>
        <Label>FAQs</Label>
        <Img height={16} width={16} source='icon-extLink' />
      </CellButton>

      <CellButton onPress={ this.props.onExternalLink.bind(this, 'guides') }
        testName='settings__external_link'>
        <Label>Guides</Label>
        <Img height={16} width={16} source='icon-extLink' />
      </CellButton>

      <CellButton onPress={ this.props.onViewSupport }
        testName='settings__view_support'>
        <Label>Contact support</Label>
        <Img height={12} width={7} source='icon-chevron' />
      </CellButton>
    </View>;
  }

  _renderQuitButton() {
    return <View style={styles.settings__footer}>
      <RedButton
        onPress={this.props.onQuit}
        testName='settings__quit'>
        <Label>Quit app</Label>
      </RedButton>
    </View>;
  }
}