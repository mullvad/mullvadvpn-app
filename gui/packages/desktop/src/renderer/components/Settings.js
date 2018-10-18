// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { ImageView, SettingsHeader, HeaderTitle } from '@mullvad/components';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import { Layout, Container } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  CloseBarItem,
  TitleBarItem,
} from './NavigationBar';
import styles from './SettingsStyles';
import { colors } from '../../config';

import type { LoginState } from '../redux/account/reducers';

type Props = {
  loginState: LoginState,
  accountExpiry: ?string,
  appVersion: string,
  consistentVersion: boolean,
  upToDateVersion: boolean,
  onQuit: () => void,
  onClose: () => void,
  onViewAccount: () => void,
  onViewSupport: () => void,
  onViewPreferences: () => void,
  onViewAdvancedSettings: () => void,
  onExternalLink: (type: string) => void,
};

export default class Settings extends Component<Props> {
  render() {
    return (
      <Layout>
        <Container>
          <View style={styles.settings}>
            <NavigationContainer>
              <NavigationBar>
                <CloseBarItem action={this.props.onClose} />
                <TitleBarItem>Settings</TitleBarItem>
              </NavigationBar>

              <View style={styles.settings__container}>
                <NavigationScrollbars style={styles.settings__scrollview}>
                  <View style={styles.settings__content}>
                    <SettingsHeader>
                      <HeaderTitle>Settings</HeaderTitle>
                    </SettingsHeader>
                    <View>
                      {this._renderTopButtons()}
                      {this._renderMiddleButtons()}
                      {this._renderBottomButtons()}
                    </View>
                    {this._renderQuitButton()}
                  </View>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  _renderTopButtons() {
    const isLoggedIn = this.props.loginState === 'ok';
    if (!isLoggedIn) {
      return null;
    }

    let isOutOfTime = false;
    let formattedExpiry = '';

    const expiryIso = this.props.accountExpiry;
    if (isLoggedIn && expiryIso) {
      const expiry = moment(expiryIso);
      isOutOfTime = expiry.isSameOrBefore(moment());
      formattedExpiry = (expiry.fromNow(true) + ' left').toUpperCase();
    }

    return (
      <View>
        <View testName="settings__account">
          {isOutOfTime ? (
            <Cell.CellButton
              onPress={this.props.onViewAccount}
              testName="settings__account_paid_until_button">
              <Cell.Label>Account</Cell.Label>
              <Cell.SubText
                testName="settings__account_paid_until_subtext"
                style={styles.settings__account_paid_until_label__error}>
                {'OUT OF TIME'}
              </Cell.SubText>
              <Cell.Icon height={12} width={7} source="icon-chevron" />
            </Cell.CellButton>
          ) : (
            <Cell.CellButton
              onPress={this.props.onViewAccount}
              testName="settings__account_paid_until_button">
              <Cell.Label>Account</Cell.Label>
              <Cell.SubText testName="settings__account_paid_until_subtext">
                {formattedExpiry}
              </Cell.SubText>
              <Cell.Icon height={12} width={7} source="icon-chevron" />
            </Cell.CellButton>
          )}
        </View>

        <Cell.CellButton onPress={this.props.onViewPreferences} testName="settings__preferences">
          <Cell.Label>Preferences</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onViewAdvancedSettings} testName="settings__advanced">
          <Cell.Label>Advanced</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
        <View style={styles.settings__cell_spacer} />
      </View>
    );
  }

  _renderMiddleButtons() {
    let icon;
    let footer;
    if (!this.props.consistentVersion || !this.props.upToDateVersion) {
      const message = !this.props.consistentVersion
        ? 'Inconsistent internal version information, please restart the app.'
        : 'Update available, download to remain safe.';

      icon = (
        <ImageView
          source="icon-alert"
          tintColor={colors.red}
          style={styles.settings__version_warning}
        />
      );
      footer = (
        <View style={styles.settings__cell_footer}>
          <Text style={styles.settings__cell_footer_label}>{message}</Text>
        </View>
      );
    } else {
      footer = <View style={styles.settings__cell_spacer} />;
    }

    return (
      <View>
        <Cell.CellButton
          onPress={this.props.onExternalLink.bind(this, 'download')}
          testName="settings__version">
          {icon}
          <Cell.Label>App version</Cell.Label>
          <Cell.SubText>{this.props.appVersion}</Cell.SubText>
          <Cell.Icon height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>
        {footer}
      </View>
    );
  }

  _renderBottomButtons() {
    return (
      <View>
        <Cell.CellButton onPress={this.props.onViewSupport} testName="settings__view_support">
          <Cell.Label>Report a problem</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton
          onPress={this.props.onExternalLink.bind(this, 'faq')}
          testName="settings__external_link">
          <Cell.Label>{'FAQs & Guides'}</Cell.Label>
          <Cell.Icon height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>
      </View>
    );
  }

  _renderQuitButton() {
    return (
      <View style={styles.settings__footer}>
        <AppButton.RedButton onPress={this.props.onQuit} testName="settings__quit">
          <AppButton.Label>Quit app</AppButton.Label>
        </AppButton.RedButton>
      </View>
    );
  }
}
