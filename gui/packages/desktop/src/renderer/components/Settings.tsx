import { HeaderTitle, ImageView, SettingsHeader } from '@mullvad/components';
import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { colors, links } from '../../config.json';
import AccountExpiry from '../lib/account-expiry';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import { Container, Layout } from './Layout';
import {
  CloseBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
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
                <TitleBarItem>Settings</TitleBarItem>
              </NavigationBar>

              <View style={styles.settings__container}>
                <NavigationScrollbars style={styles.settings__scrollview}>
                  <View style={styles.settings__content}>
                    <SettingsHeader>
                      <HeaderTitle>Settings</HeaderTitle>
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
        <AppButton.RedButton onPress={this.props.onQuit}>{'Quit app'}</AppButton.RedButton>
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

    return (
      <View>
        <View>
          {isOutOfTime ? (
            <Cell.CellButton onPress={this.props.onViewAccount}>
              <Cell.Label>Account</Cell.Label>
              <Cell.SubText style={styles.settings__account_paid_until_label__error}>
                {'OUT OF TIME'}
              </Cell.SubText>
              <Cell.Icon height={12} width={7} source="icon-chevron" />
            </Cell.CellButton>
          ) : (
            <Cell.CellButton onPress={this.props.onViewAccount}>
              <Cell.Label>Account</Cell.Label>
              <Cell.SubText>{formattedExpiry}</Cell.SubText>
              <Cell.Icon height={12} width={7} source="icon-chevron" />
            </Cell.CellButton>
          )}
        </View>

        <Cell.CellButton onPress={this.props.onViewPreferences}>
          <Cell.Label>Preferences</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onViewAdvancedSettings}>
          <Cell.Label>Advanced</Cell.Label>
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
        <Cell.CellButton disabled={this.props.isOffline} onPress={this.openDownloadLink}>
          {icon}
          <Cell.Label>App version</Cell.Label>
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
          <Cell.Label>Report a problem</Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton disabled={this.props.isOffline} onPress={this.openFaqLink}>
          <Cell.Label>{'FAQs & Guides'}</Cell.Label>
          <Cell.Icon height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>
      </View>
    );
  }
}
