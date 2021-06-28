import * as React from 'react';
import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { AccountToken, TunnelState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { LoginState } from '../redux/account/reducers';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import * as Cell from './cell';
import {
  StyledAccountTokenContainer,
  StyledAccountTokenLabel,
  StyledAccountTokenMessage,
  StyledBody,
  StyledBuyCreditButton,
  StyledContainer,
  StyledCustomScrollbars,
  StyledDisconnectButton,
  StyledFooter,
  StyledHeader,
  StyledMessage,
  StyledModalCellContainer,
  StyledStatusIcon,
  StyledTitle,
} from './ExpiredAccountErrorViewStyles';
import { calculateHeaderBarStyle, HeaderBarStyle } from './HeaderBar';
import ImageView from './ImageView';
import { Layout } from './Layout';
import { ModalAlert, ModalAlertType, ModalContainer, ModalMessage } from './Modal';

export enum RecoveryAction {
  openBrowser,
  disconnect,
  disableBlockedWhenDisconnected,
}

interface IExpiredAccountErrorViewProps {
  isBlocked: boolean;
  blockWhenDisconnected: boolean;
  accountToken?: AccountToken;
  loginState: LoginState;
  tunnelState: TunnelState;
  onExternalLinkWithAuth: (url: string) => Promise<void>;
  onDisconnect: () => Promise<void>;
  setBlockWhenDisconnected: (value: boolean) => void;
  navigateToRedeemVoucher: () => void;
}

interface IExpiredAccountErrorViewState {
  showBlockWhenDisconnectedAlert: boolean;
}

export default class ExpiredAccountErrorView extends React.Component<
  IExpiredAccountErrorViewProps,
  IExpiredAccountErrorViewState
> {
  public state: IExpiredAccountErrorViewState = {
    showBlockWhenDisconnectedAlert: false,
  };

  public render() {
    const headerBarStyle =
      this.props.loginState.type === 'ok' && this.props.loginState.method === 'new_account'
        ? HeaderBarStyle.default
        : calculateHeaderBarStyle(this.props.tunnelState);

    return (
      <ModalContainer>
        <Layout>
          <StyledHeader barStyle={headerBarStyle} />
          <StyledCustomScrollbars fillContainer>
            <StyledContainer>
              <StyledBody>{this.renderContent()}</StyledBody>

              <StyledFooter>
                {this.getRecoveryAction() === RecoveryAction.disconnect && (
                  <AppButton.BlockingButton onClick={this.props.onDisconnect}>
                    <StyledDisconnectButton>
                      {messages.pgettext('connect-view', 'Disconnect')}
                    </StyledDisconnectButton>
                  </AppButton.BlockingButton>
                )}

                {this.renderExternalPaymentButton()}

                <AppButton.GreenButton
                  disabled={this.getRecoveryAction() === RecoveryAction.disconnect}
                  onClick={this.props.navigateToRedeemVoucher}>
                  {messages.pgettext('connect-view', 'Redeem voucher')}
                </AppButton.GreenButton>
              </StyledFooter>

              {this.state.showBlockWhenDisconnectedAlert && this.renderBlockWhenDisconnectedAlert()}
            </StyledContainer>
          </StyledCustomScrollbars>
        </Layout>
      </ModalContainer>
    );
  }

  private renderContent() {
    if (this.isNewAccount()) {
      return this.renderWelcomeView();
    }

    return (
      <>
        <StyledStatusIcon>
          <ImageView source="icon-fail" height={60} width={60} />
        </StyledStatusIcon>
        <StyledTitle>{messages.pgettext('connect-view', 'Out of time')}</StyledTitle>
        <StyledMessage>
          {sprintf('%(introduction)s %(recoveryMessage)s', {
            introduction: messages.pgettext(
              'connect-view',
              'You have no more VPN time left on this account.',
            ),
            recoveryMessage: this.getRecoveryActionMessage(),
          })}
        </StyledMessage>
      </>
    );
  }

  private renderWelcomeView() {
    return (
      <>
        <StyledTitle>{messages.pgettext('connect-view', 'Congrats!')}</StyledTitle>
        <StyledAccountTokenMessage>
          {messages.pgettext('connect-view', 'Here’s your account number. Save it!')}
          <StyledAccountTokenContainer>
            <StyledAccountTokenLabel accountToken={this.props.accountToken || ''} />
          </StyledAccountTokenContainer>
        </StyledAccountTokenMessage>

        <StyledMessage>
          {sprintf('%(introduction)s %(recoveryMessage)s', {
            introduction: messages.pgettext(
              'connect-view',
              'To start using the app, you first need to add time to your account.',
            ),
            recoveryMessage: this.getRecoveryActionMessage(),
          })}
        </StyledMessage>
      </>
    );
  }

  private getRecoveryActionMessage() {
    switch (this.getRecoveryAction()) {
      case RecoveryAction.openBrowser:
      case RecoveryAction.disableBlockedWhenDisconnected:
        return messages.pgettext(
          'connect-view',
          'Either buy credit on our website or redeem a voucher.',
        );
      case RecoveryAction.disconnect:
        return messages.pgettext(
          'connect-view',
          'To add more, you will need to disconnect and access the Internet with an unsecure connection.',
        );
    }
  }

  private renderExternalPaymentButton() {
    const buttonText = this.isNewAccount()
      ? messages.gettext('Buy credit')
      : messages.gettext('Buy more credit');

    return (
      <AppButton.BlockingButton
        disabled={this.getRecoveryAction() === RecoveryAction.disconnect}
        onClick={this.onOpenExternalPayment}>
        <AriaDescriptionGroup>
          <AriaDescribed>
            <StyledBuyCreditButton>
              <AppButton.Label>{buttonText}</AppButton.Label>
              <AriaDescription>
                <AppButton.Icon
                  source="icon-extLink"
                  height={16}
                  width={16}
                  aria-label={messages.pgettext('accessibility', 'Opens externally')}
                />
              </AriaDescription>
            </StyledBuyCreditButton>
          </AriaDescribed>
        </AriaDescriptionGroup>
      </AppButton.BlockingButton>
    );
  }

  private renderBlockWhenDisconnectedAlert() {
    return (
      <ModalAlert
        type={ModalAlertType.info}
        buttons={[
          <AppButton.BlueButton
            key="cancel"
            onClick={this.onCloseBlockWhenDisconnectedInstructions}>
            {messages.gettext('Close')}
          </AppButton.BlueButton>,
        ]}
        close={this.onCloseBlockWhenDisconnectedInstructions}>
        <ModalMessage>
          {messages.pgettext(
            'connect-view',
            'You need to disable "Always require VPN" in order to access the Internet to add time.',
          )}
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'connect-view',
            'Remember, turning it off will allow network traffic while the VPN is disconnected until you turn it back on under Advanced settings.',
          )}
        </ModalMessage>
        <StyledModalCellContainer>
          <Cell.Label>{messages.pgettext('connect-view', 'Always require VPN')}</Cell.Label>
          <Cell.Switch
            isOn={this.props.blockWhenDisconnected}
            onChange={this.props.setBlockWhenDisconnected}
          />
        </StyledModalCellContainer>
      </ModalAlert>
    );
  }

  private isNewAccount() {
    return this.props.loginState.type === 'ok' && this.props.loginState.method === 'new_account';
  }

  private onOpenExternalPayment = async (): Promise<void> => {
    if (this.getRecoveryAction() === RecoveryAction.disableBlockedWhenDisconnected) {
      this.setState({ showBlockWhenDisconnectedAlert: true });
    } else {
      await this.props.onExternalLinkWithAuth(links.purchase);
    }
  };

  private getRecoveryAction() {
    const { blockWhenDisconnected, isBlocked } = this.props;

    if (blockWhenDisconnected && isBlocked) {
      return RecoveryAction.disableBlockedWhenDisconnected;
    } else if (!blockWhenDisconnected && isBlocked) {
      return RecoveryAction.disconnect;
    } else {
      return RecoveryAction.openBrowser;
    }
  }

  private onCloseBlockWhenDisconnectedInstructions = () => {
    this.setState({ showBlockWhenDisconnectedAlert: false });
  };
}
