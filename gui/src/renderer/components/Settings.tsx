import * as React from 'react';

import { colors, links } from '../../config.json';
import { formatRemainingTime, hasExpired } from '../../shared/account-expiry';
import { messages } from '../../shared/gettext';
import History from '../lib/history';
import { LoginState } from '../redux/account/reducers';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import * as Cell from './cell';
import { BackAction } from './KeyboardNavigation';
import { Layout } from './Layout';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import {
  StyledCellIcon,
  StyledCellSpacer,
  StyledContainer,
  StyledContent,
  StyledNavigationScrollbars,
  StyledOutOfTimeSubText,
  StyledQuitButton,
  StyledSettingsContent,
} from './SettingsStyles';

export interface IProps {
  preferredLocaleDisplayName: string;
  loginState: LoginState;
  connectedToDaemon: boolean;
  accountExpiry?: string;
  appVersion: string;
  consistentVersion: boolean;
  upToDateVersion: boolean;
  suggestedIsBeta: boolean;
  isOffline: boolean;
  onQuit: () => void;
  onClose: () => void;
  onViewSelectLanguage: () => void;
  onViewAccount: () => void;
  onViewSupport: () => void;
  onViewPreferences: () => void;
  onViewAdvancedSettings: () => void;
  onExternalLink: (url: string) => void;
  updateAccountData: () => void;
  history: History;
}

export default class Settings extends React.Component<IProps> {
  public componentDidMount() {
    if (this.props.history.action === 'PUSH') {
      this.props.updateAccountData();
    }
  }

  public render() {
    const showLargeTitle = this.props.loginState.type !== 'ok';

    return (
      <BackAction icon="close" action={this.props.onClose}>
        <Layout>
          <StyledContainer>
            <NavigationContainer>
              <NavigationBar alwaysDisplayBarTitle={!showLargeTitle}>
                <NavigationItems>
                  <TitleBarItem>
                    {
                      // TRANSLATORS: Title label in navigation bar
                      messages.pgettext('navigation-bar', 'Settings')
                    }
                  </TitleBarItem>
                </NavigationItems>
              </NavigationBar>

              <StyledNavigationScrollbars fillContainer>
                <StyledContent>
                  {showLargeTitle && (
                    <SettingsHeader>
                      <HeaderTitle>{messages.pgettext('navigation-bar', 'Settings')}</HeaderTitle>
                    </SettingsHeader>
                  )}

                  <StyledSettingsContent>
                    {this.renderTopButtons()}
                    {this.renderMiddleButtons()}
                    {this.renderBottomButtons()}
                  </StyledSettingsContent>
                </StyledContent>

                {this.renderQuitButton()}
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </StyledContainer>
        </Layout>
      </BackAction>
    );
  }

  private openDownloadLink = () =>
    this.props.onExternalLink(this.props.suggestedIsBeta ? links.betaDownload : links.download);
  private openFaqLink = () => this.props.onExternalLink(links.faq);

  private renderQuitButton() {
    return (
      <StyledQuitButton onClick={this.props.onQuit}>
        {messages.pgettext('settings-view', 'Quit app')}
      </StyledQuitButton>
    );
  }

  private renderTopButtons() {
    const isLoggedIn = this.props.loginState.type === 'ok';
    if (!isLoggedIn || !this.props.connectedToDaemon) {
      return null;
    }

    const isOutOfTime = this.props.accountExpiry ? hasExpired(this.props.accountExpiry) : false;
    const formattedExpiry = this.props.accountExpiry
      ? formatRemainingTime(this.props.accountExpiry).toUpperCase()
      : '';

    const outOfTimeMessage = messages.pgettext('settings-view', 'OUT OF TIME');

    return (
      <>
        <Cell.CellButton onClick={this.props.onViewAccount}>
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Account' view
              messages.pgettext('settings-view', 'Account')
            }
          </Cell.Label>
          <StyledOutOfTimeSubText isOutOfTime={isOutOfTime}>
            {isOutOfTime ? outOfTimeMessage : formattedExpiry}
          </StyledOutOfTimeSubText>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton onClick={this.props.onViewPreferences}>
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Preferences' view
              messages.pgettext('settings-view', 'Preferences')
            }
          </Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton onClick={this.props.onViewAdvancedSettings}>
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Advanced' settings view
              messages.pgettext('settings-view', 'Advanced')
            }
          </Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
        <StyledCellSpacer />
      </>
    );
  }

  private renderMiddleButtons() {
    let icon;
    let footer;
    if (!this.props.consistentVersion || !this.props.upToDateVersion) {
      const inconsistentVersionMessage = messages.pgettext(
        'settings-view',
        'App is out of sync. Please quit and restart.',
      );

      const updateAvailableMessage = messages.pgettext(
        'settings-view',
        'Update available. Install the latest app version to stay up to date.',
      );

      const message = !this.props.consistentVersion
        ? inconsistentVersionMessage
        : updateAvailableMessage;

      icon = <StyledCellIcon source="icon-alert" width={18} tintColor={colors.red} />;
      footer = (
        <Cell.Footer>
          <Cell.FooterText>{message}</Cell.FooterText>
        </Cell.Footer>
      );
    } else {
      footer = <StyledCellSpacer />;
    }

    return (
      <AriaDescriptionGroup>
        <AriaDescribed>
          <Cell.CellButton disabled={this.props.isOffline} onClick={this.openDownloadLink}>
            {icon}
            <Cell.Label>{messages.pgettext('settings-view', 'App version')}</Cell.Label>
            <Cell.SubText>{this.props.appVersion}</Cell.SubText>
            <AriaDescription>
              <Cell.Icon
                height={16}
                width={16}
                source="icon-extLink"
                aria-label={messages.pgettext('accessibility', 'Opens externally')}
              />
            </AriaDescription>
          </Cell.CellButton>
        </AriaDescribed>
        {footer}
      </AriaDescriptionGroup>
    );
  }

  private renderBottomButtons() {
    return (
      <>
        <Cell.CellButton onClick={this.props.onViewSupport}>
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Report a problem' help view
              messages.pgettext('settings-view', 'Report a problem')
            }
          </Cell.Label>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <AriaDescriptionGroup>
          <AriaDescribed>
            <Cell.CellButton disabled={this.props.isOffline} onClick={this.openFaqLink}>
              <Cell.Label>
                {
                  // TRANSLATORS: Link to the webpage
                  messages.pgettext('settings-view', 'FAQs & Guides')
                }
              </Cell.Label>
              <AriaDescription>
                <Cell.Icon
                  height={16}
                  width={16}
                  source="icon-extLink"
                  aria-label={messages.pgettext('accessibility', 'Opens externally')}
                />
              </AriaDescription>
            </Cell.CellButton>
          </AriaDescribed>
        </AriaDescriptionGroup>

        <Cell.CellButton onClick={this.props.onViewSelectLanguage}>
          <StyledCellIcon width={24} height={24} source="icon-language" />
          <Cell.Label>
            {
              // TRANSLATORS: Navigation button to the 'Language' settings view
              messages.pgettext('settings-view', 'Language')
            }
          </Cell.Label>
          <Cell.SubText>{this.props.preferredLocaleDisplayName}</Cell.SubText>
          <Cell.Icon height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
      </>
    );
  }
}
