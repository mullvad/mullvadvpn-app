import * as React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import ConnectionPanelContainer from '../containers/ConnectionPanelContainer';
import * as AppButton from './AppButton';
import { bigText } from './common-styles';
import ImageView from './ImageView';
import Marquee from './Marquee';
import { MultiButton } from './MultiButton';
import SecuredLabel, { SecuredDisplayStyle } from './SecuredLabel';

interface ITunnelControlProps {
  tunnelState: TunnelState;
  blockWhenDisconnected: boolean;
  selectedRelayName: string;
  city?: string;
  country?: string;
  onConnect: () => void;
  onDisconnect: () => void;
  onReconnect: () => void;
  onSelectLocation: () => void;
}

const SwitchLocationButton = styled(AppButton.TransparentButton)({
  marginBottom: '18px',
});

const Secured = styled(SecuredLabel)({
  fontFamily: 'Open Sans',
  fontSize: '16px',
  fontWeight: 800,
  lineHeight: '22px',
  marginBottom: '2px',
});

const Footer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  paddingBottom: '22px',
  paddingLeft: '22px',
  paddingRight: '22px',
});

const Body = styled.div({
  display: 'flex',
  flexDirection: 'column',
  padding: '0 22px',
  marginTop: '176px',
  flex: 1,
});

const Wrapper = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

const Location = styled.div({
  display: 'flex',
  flexDirection: 'column',
  marginBottom: 2,
});

const StyledMarquee = styled(Marquee)({
  ...bigText,
  overflow: 'hidden',
});

const SelectedLocationChevron = styled(AppButton.Icon)({
  margin: '0 4px',
});

export default class TunnelControl extends React.Component<ITunnelControlProps> {
  public render() {
    let state = this.props.tunnelState.state;

    switch (this.props.tunnelState.state) {
      case 'disconnecting':
        switch (this.props.tunnelState.details) {
          case 'block':
            state = 'error';
            break;
          case 'reconnect':
            state = 'connecting';
            break;
          default:
            state = 'disconnecting';
            break;
        }
        break;
    }

    switch (state) {
      case 'connecting':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.securing} />
              <Location>
                {this.renderCity()}
                {this.renderCountry()}
              </Location>
              <ConnectionPanelContainer />
            </Body>
            <Footer>
              {this.switchLocationButton()}
              <MultiButton mainButton={this.cancelButton} sideButton={this.reconnectButton} />
            </Footer>
          </Wrapper>
        );
      case 'connected':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.secured} />
              <Location>
                {this.renderCity()}
                {this.renderCountry()}
              </Location>
              <ConnectionPanelContainer />
            </Body>
            <Footer>
              {this.switchLocationButton()}
              <MultiButton mainButton={this.disconnectButton} sideButton={this.reconnectButton} />
            </Footer>
          </Wrapper>
        );

      case 'error':
        if (
          this.props.tunnelState.state === 'error' &&
          this.props.tunnelState.details.blockFailure
        ) {
          return (
            <Wrapper>
              <Body>
                <Secured displayStyle={SecuredDisplayStyle.failedToSecure} />
              </Body>
              <Footer>
                {this.switchLocationButton()}
                <MultiButton mainButton={this.dismissButton} sideButton={this.reconnectButton} />
              </Footer>
            </Wrapper>
          );
        } else {
          return (
            <Wrapper>
              <Body>
                <Secured displayStyle={SecuredDisplayStyle.blocked} />
              </Body>
              <Footer>
                {this.switchLocationButton()}
                <MultiButton mainButton={this.cancelButton} sideButton={this.reconnectButton} />
              </Footer>
            </Wrapper>
          );
        }

      case 'disconnecting':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.secured} />
              <Location>{this.renderCountry()}</Location>
            </Body>
            <Footer>
              {this.selectLocationButton()}
              {this.connectButton()}
            </Footer>
          </Wrapper>
        );

      case 'disconnected': {
        const displayStyle = this.props.blockWhenDisconnected
          ? SecuredDisplayStyle.blocked
          : SecuredDisplayStyle.unsecured;
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={displayStyle} />
              <Location>{this.renderCountry()}</Location>
            </Body>
            <Footer>
              {this.selectLocationButton()}
              {this.connectButton()}
            </Footer>
          </Wrapper>
        );
      }

      default:
        throw new Error(`Unknown TunnelState: ${this.props.tunnelState}`);
    }
  }

  private renderCity() {
    return <StyledMarquee>{this.props.city}</StyledMarquee>;
  }

  private renderCountry() {
    return <StyledMarquee>{this.props.country}</StyledMarquee>;
  }

  private switchLocationButton() {
    return (
      <SwitchLocationButton onClick={this.props.onSelectLocation}>
        {messages.pgettext('tunnel-control', 'Switch location')}
      </SwitchLocationButton>
    );
  }

  private selectLocationButton() {
    return (
      <SwitchLocationButton
        onClick={this.props.onSelectLocation}
        aria-label={sprintf(
          messages.pgettext('accessibility', 'Select location. Current location is %(location)s'),
          { location: this.props.selectedRelayName },
        )}>
        <AppButton.Label>{this.props.selectedRelayName}</AppButton.Label>
        <SelectedLocationChevron height={12} width={7} source="icon-chevron" />
      </SwitchLocationButton>
    );
  }

  private connectButton() {
    return (
      <AppButton.GreenButton onClick={this.props.onConnect}>
        {messages.pgettext('tunnel-control', 'Secure my connection')}
      </AppButton.GreenButton>
    );
  }

  private disconnectButton = (props: AppButton.IProps) => {
    return (
      <AppButton.RedTransparentButton onClick={this.props.onDisconnect} {...props}>
        {messages.gettext('Disconnect')}
      </AppButton.RedTransparentButton>
    );
  };

  private cancelButton = (props: AppButton.IProps) => {
    return (
      <AppButton.RedTransparentButton onClick={this.props.onDisconnect} {...props}>
        {messages.gettext('Cancel')}
      </AppButton.RedTransparentButton>
    );
  };

  private dismissButton = (props: AppButton.IProps) => {
    return (
      <AppButton.RedTransparentButton onClick={this.props.onDisconnect} {...props}>
        {messages.gettext('Dismiss')}
      </AppButton.RedTransparentButton>
    );
  };

  private reconnectButton = (props: AppButton.IProps) => {
    return (
      <AppButton.RedTransparentButton
        onClick={this.props.onReconnect}
        aria-label={messages.gettext('Reconnect')}
        {...props}>
        <ImageView height={22} width={22} source="icon-reload" tintColor="white" />
      </AppButton.RedTransparentButton>
    );
  };
}
