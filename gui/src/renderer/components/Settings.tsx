import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { colors, links } from '../../config.json';
import AccountExpiry from '../../shared/account-expiry';
import { messages } from '../../shared/gettext';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import { Container, Layout } from './Layout';
import {
  CloseBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import styles from './SettingsStyles';

import { LoginState } from '../redux/account/reducers';

export interface IProps {
  preferredLocaleDisplayName: string;
  loginState: LoginState;
  accountExpiry?: string;
  expiryLocale: string;
  appVersion: string;
  consistentVersion: boolean;
  upToDateVersion: boolean;
  isOffline: boolean;
  onQuit: () => void;
  onClose: () => void;
  onViewSelectLanguage: () => void;
  onViewAccount: () => void;
  onViewSupport: () => void;
  onViewPreferences: () => void;
  onViewAdvancedSettings: () => void;
  onExternalLink: (url: string) => void;
}

export default class Settings extends Component<IProps> {
  public render() {
    const showLargeTitle = this.props.loginState.type !== 'ok';

    return (
      <Layout>
        <Container>
          <View style={styles.settings}>
            <NavigationContainer>
              <NavigationBar alwaysDisplayBarTitle={!showLargeTitle}>
                <NavigationItems>
                  <CloseBarItem action={this.props.onClose} />
                  <TitleBarItem>
                    {
                      // TRANSLATORS: Title label in navigation bar
                      messages.pgettext('navigation-bar', 'Settings')
                    }
                  </TitleBarItem>
                </NavigationItems>
              </NavigationBar>

              <View style={styles.container}>
                <NavigationScrollbars style={styles.scrollview}>
                  <View style={styles.content}>
                    {showLargeTitle && (
                      <SettingsHeader>
                        <HeaderTitle>{messages.pgettext('navigation-bar', 'Settings')}</HeaderTitle>
                      </SettingsHeader>
                    )}
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

  private openDownloadLink = () => this.props.onExternalLink(links.download);
  private openFaqLink = () => this.props.onExternalLink(links.faq);

  private renderQuitButton() {
    return (
      <View style={styles.quitButtonFooter}>
        <AppButton.RedButton onPress={this.props.onQuit}>
          {messages.pgettext('settings-view', 'Quit app')}
        </AppButton.RedButton>
      </View>
    );
  }

  private renderTopButtons() {
    const isLoggedIn = this.props.loginState.type === 'ok';
    if (!isLoggedIn) {
      return null;
    }

    const expiry = this.props.accountExpiry
      ? new AccountExpiry(this.props.accountExpiry, this.props.expiryLocale)
      : null;
    const isOutOfTime = expiry ? expiry.hasExpired() : false;
    const formattedExpiry = expiry ? expiry.remainingTime().toUpperCase() : '';

    const outOfTimeMessage = messages.pgettext('settings-view', 'OUT OF TIME');

    return (
      <View>
        <View>
          <Cell.CellButton onPress={this.props.onViewAccount}>
            <Cell.Label>
              {
                // TRANSLATORS: Navigation button to the 'Account' view
                messages.pgettext('settings-view', 'Account')
              }
            </Cell.Label>
            <Cell.SubText style={isOutOfTime ? styles.accountPaidUntilErrorLabel : undefined}>
              {isOutOfTime ? outOfTimeMessage : formattedExpiry}
            </Cell.SubText>
            <Cell.Icon height={12} width={7} source="icon-chevron" />
          </Cell.CellButton>
        </View>

        <Cell.CellButton onPress={this.props.onViewPreferences}>
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Preferences' view
              messages.pgettext('settings-view', 'Preferences')
            }
          </Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onViewAdvancedSettings}>
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Advanced' settings view
              messages.pgettext('settings-view', 'Advanced')
            }
          </Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
        <View style={styles.cellSpacer} />
      </View>
    );
  }

  private renderMiddleButtons() {
    let icon;
    let footer;
    if (!this.props.consistentVersion || !this.props.upToDateVersion) {
      const inconsistentVersionMessage = messages.pgettext(
        'settings-view',
        'Inconsistent internal version information, please restart the app.',
      );

      const updateAvailableMessage = messages.pgettext(
        'settings-view',
        'Update available, download to remain safe.',
      );

      const message = !this.props.consistentVersion
        ? inconsistentVersionMessage
        : updateAvailableMessage;

      icon = <Cell.UntintedIcon source="icon-alert" tintColor={colors.red} />;
      footer = (
        <View style={styles.cellFooter}>
          <Text style={styles.cellFooterLabel}>{message}</Text>
        </View>
      );
    } else {
      footer = <View style={styles.cellSpacer} />;
    }

    return (
      <View>
        <Cell.CellButton disabled={this.props.isOffline} onPress={this.openDownloadLink}>
          {icon}
          <Cell.Label>{messages.pgettext('settings-view', 'App version')}</Cell.Label>
          <Cell.SubText style={styles.appVersionLabel}>{this.props.appVersion}</Cell.SubText>
          <Cell.Icon height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>
        {footer}
      </View>
    );
  }

  private renderBottomButtons() {
    return (
      <View>
        <Cell.CellButton onPress={this.props.onViewSupport}>
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Report a problem' help view
              messages.pgettext('settings-view', 'Report a problem')
            }
          </Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton disabled={this.props.isOffline} onPress={this.openFaqLink}>
          <Cell.Label>
            {
              // TRANSLATORS: Link to the webpage
              messages.pgettext('settings-view', 'FAQs & Guides')
            }
          </Cell.Label>
          <Cell.Icon height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onViewSelectLanguage}>
          <Cell.UntintedIcon width={24} height={24} source="icon-language" />
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Language' settings view
              messages.pgettext('settings-view', 'Language')
            }
          </Cell.Label>
          <Cell.SubText>{this.props.preferredLocaleDisplayName}</Cell.SubText>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
      </View>
    );
  }
}
