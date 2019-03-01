import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { colors, links } from '../../config.json';
import { pgettext } from '../../shared/gettext';
import AccountExpiry from '../lib/account-expiry';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import {
  CloseBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import styles from './SettingsStyles';

import { LoginState } from '../redux/account/reducers';

export interface IProps {
  loginState: LoginState;
  accountExpiry?: string;
  appVersion: string;
  consistentVersion: boolean;
  upToDateVersion: boolean;
  isOffline: boolean;
  onQuit: () => void;
  onClose: () => void;
  onViewAccount: () => void;
  onViewSupport: () => void;
  onViewPreferences: () => void;
  onViewAdvancedSettings: () => void;
  onExternalLink: (url: string) => void;
}

export default class Settings extends Component<IProps> {
  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.settings}>
            <NavigationContainer>
              <NavigationBar>
                <CloseBarItem action={this.props.onClose} />
                <TitleBarItem>
                  {// TRANSLATORS: Title label in navigation bar
                  pgettext('settings-view-nav', 'Settings')}
                </TitleBarItem>
              </NavigationBar>

              <View style={styles.settings__container}>
                <NavigationScrollbars style={styles.settings__scrollview}>
                  <View style={styles.settings__content}>
                    <SettingsHeader>
                      <HeaderTitle>{pgettext('settings-view', 'Settings')}</HeaderTitle>
                    </SettingsHeader>
                    <View>
                      {this.renderTopButtons()}
                      {this.renderMiddleButtons()}
                      {this.renderBottomButtons()}
                    </View>
                    {this.renderQuitButton()}
                  </View>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  private renderQuitButton() {
    return (
      <View style={styles.settings__footer}>
        <AppButton.RedButton onPress={this.props.onQuit}>
          {pgettext('settings-view', 'Quit app')}
        </AppButton.RedButton>
      </View>
    );
  }

  private renderTopButtons() {
    const isLoggedIn = this.props.loginState === 'ok';
    if (!isLoggedIn) {
      return null;
    }

    const expiry = this.props.accountExpiry ? new AccountExpiry(this.props.accountExpiry) : null;
    const isOutOfTime = expiry ? expiry.hasExpired() : false;
    const formattedExpiry = expiry ? expiry.remainingTime().toUpperCase() : '';

    const outOfTimeMessage = pgettext('settings-view', 'OUT OF TIME');

    return (
      <View>
        <View>
          <Cell.CellButton onPress={this.props.onViewAccount}>
            <Cell.Label>{pgettext('settings-view', 'Account')}</Cell.Label>
            <Cell.SubText style={styles.settings__account_paid_until_label__error}>
              {isOutOfTime ? outOfTimeMessage : formattedExpiry}
            </Cell.SubText>
            <Cell.Icon height={12} width={7} source="icon-chevron" />
          </Cell.CellButton>
        </View>

        <Cell.CellButton onPress={this.props.onViewPreferences}>
          <Cell.Label>{pgettext('settings-view', 'Preferences')}</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onViewAdvancedSettings}>
          <Cell.Label>{pgettext('settings-view', 'Advanced')}</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
        <View style={styles.settings__cell_spacer} />
      </View>
    );
  }

  private renderMiddleButtons() {
    let icon;
    let footer;
    if (!this.props.consistentVersion || !this.props.upToDateVersion) {
      const inconsistentVersionMessage = pgettext(
        'settings-view',
        'Inconsistent internal version information, please restart the app.',
      );

      const updateAvailableMessage = pgettext(
        'settings-view',
        'Update available, download to remain safe.',
      );

      const message = !this.props.consistentVersion
        ? inconsistentVersionMessage
        : updateAvailableMessage;

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
        <Cell.CellButton disabled={this.props.isOffline} onPress={this.openDownloadLink}>
          {icon}
          <Cell.Label>{pgettext('settings-view', 'App version')}</Cell.Label>
          <Cell.SubText>{this.props.appVersion}</Cell.SubText>
          <Cell.Icon height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>
        {footer}
      </View>
    );
  }

  private openDownloadLink = () => this.props.onExternalLink(links.download);
  private openFaqLink = () => this.props.onExternalLink(links.faq);

  private renderBottomButtons() {
    return (
      <View>
        <Cell.CellButton onPress={this.props.onViewSupport}>
          <Cell.Label>{pgettext('settings-view', 'Report a problem')}</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton disabled={this.props.isOffline} onPress={this.openFaqLink}>
          <Cell.Label>{pgettext('settings-view', 'FAQs & Guides')}</Cell.Label>
          <Cell.Icon height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>
      </View>
    );
  }
}
