import * as React from 'react';
import { Component, Styles, View } from 'reactxp';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import * as AppButton from './AppButton';
import ImageView from './ImageView';

export enum RecoveryAction {
  openBrowser,
  disconnectAndOpenBrowser,
  disableBlockedWhenDisconnected,
}

interface IProps {
  isBlocked: boolean;
  blockWhenDisconnected: boolean;
  action: (recoveryAction: RecoveryAction) => void;
}

interface IState {
  recoveryAction: RecoveryAction;
}

const styles = {
  container: Styles.createViewStyle({
    flex: 1,
    paddingTop: 94,
  }),
  body: Styles.createViewStyle({
    flex: 1,
    paddingHorizontal: 24,
  }),
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 32,
    fontWeight: '900',
    lineHeight: 40,
    color: colors.white,
    marginBottom: 8,
  }),
  message: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    lineHeight: 20,
    fontWeight: '600',
    color: colors.white,
    marginBottom: 24,
  }),
  statusIcon: Styles.createViewStyle({
    alignSelf: 'center',
    width: 60,
    height: 60,
    marginBottom: 32,
  }),
};

export default class ExpiredAccountErrorView extends Component<IProps, IState> {
  public static getDerivedStateFromProps(props: IProps): IState {
    const { blockWhenDisconnected, isBlocked } = props;

    if (blockWhenDisconnected && isBlocked) {
      return { recoveryAction: RecoveryAction.disableBlockedWhenDisconnected };
    } else if (!blockWhenDisconnected && isBlocked) {
      return { recoveryAction: RecoveryAction.disconnectAndOpenBrowser };
    } else {
      return { recoveryAction: RecoveryAction.openBrowser };
    }
  }
  public state: IState = { recoveryAction: RecoveryAction.openBrowser };

  public render() {
    return (
      <View style={styles.container}>
        <View style={styles.statusIcon}>
          <ImageView source="icon-fail" height={60} width={60} />
        </View>
        <View style={styles.body}>
          <View style={styles.title}>{messages.pgettext('connect-view', 'Out of time')}</View>
          {this.renderContent()}
        </View>
      </View>
    );
  }

  private renderContent() {
    switch (this.state.recoveryAction) {
      case RecoveryAction.disconnectAndOpenBrowser:
        return <DisconnectAndOpenBrowserContentView actionHandler={this.handleAction} />;
      case RecoveryAction.openBrowser:
        return <OpenBrowserContentView actionHandler={this.handleAction} />;
      case RecoveryAction.disableBlockedWhenDisconnected:
        return <DisableBlockWhenDisconnectedContentView />;
    }
  }

  private handleAction = () => {
    this.props.action(this.state.recoveryAction);
  };
}

class DisconnectAndOpenBrowserContentView extends Component<{ actionHandler: () => void }> {
  public render() {
    return (
      <View>
        <View style={styles.message}>
          {messages.pgettext(
            'connect-view',
            'You have no more VPN time left on this account. To buy more credit on our website, you will need to access the Internet with an unsecured connection.',
          )}
        </View>
        <View>
          <AppButton.RedButton onPress={this.props.actionHandler}>
            <AppButton.Label>
              {messages.pgettext('connect-view', 'Disconnect and buy more credit')}
            </AppButton.Label>
            <AppButton.Icon source="icon-extLink" height={16} width={16} />
          </AppButton.RedButton>
        </View>
      </View>
    );
  }
}

class OpenBrowserContentView extends Component<{ actionHandler: () => void }> {
  public render() {
    return (
      <View>
        <View style={styles.message}>
          {messages.pgettext(
            'connect-view',
            'You have no more VPN time left on this account. Please log in on our website to buy more credit.',
          )}
        </View>
        <View>
          <AppButton.GreenButton onPress={this.props.actionHandler}>
            <AppButton.Label>
              {messages.pgettext('connect-view', 'Buy more credit')}
            </AppButton.Label>
            <AppButton.Icon source="icon-extLink" height={16} width={16} />
          </AppButton.GreenButton>
        </View>
      </View>
    );
  }
}

class DisableBlockWhenDisconnectedContentView extends Component {
  public render() {
    return (
      <View>
        <View style={styles.message}>
          {messages.pgettext(
            'connect-view',
            'You have no more VPN time left on this account. Before you can buy more credit on our website, you first need to turn off the app\'s "Block when disconnected" option under Advanced settings.',
          )}
        </View>
        <View>
          <AppButton.GreenButton disabled={true}>
            <AppButton.Label>
              {messages.pgettext('connect-view', 'Buy more credit')}
            </AppButton.Label>
            <AppButton.Icon source="icon-extLink" height={16} width={16} />
          </AppButton.GreenButton>
        </View>
      </View>
    );
  }
}
