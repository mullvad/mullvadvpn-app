import * as React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { TunnelState } from '../../shared/daemon-rpc-types';
import { messages, relayLocations } from '../../shared/gettext';
import ConnectionPanelContainer from '../containers/ConnectionPanelContainer';
import * as AppButton from './AppButton';
import { hugeText, measurements, normalText } from './common-styles';
import ImageView from './ImageView';
import { Footer } from './Layout';
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

const Secured = styled(SecuredLabel)(normalText, {
  fontWeight: 700,
  lineHeight: '22px',
});

const Body = styled.div({
  display: 'flex',
  flexDirection: 'column',
  padding: `0 ${measurements.viewMargin}`,
  minHeight: '185px',
});

const Wrapper = styled.div({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'end',
  flex: 1,
});

const Location = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

const LocationRow = styled.div({
  height: '36px',
});

const StyledMarquee = styled(Marquee)(hugeText, {
  lineHeight: '36px',
  overflow: 'hidden',
});

const SelectedLocationChevron = styled(AppButton.Icon)({
  margin: '0 4px',
});

export default class TunnelControl extends React.Component<ITunnelControlProps> {
  public render() {
    let state = this.props.tunnelState.state;
    let pq = false;

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
      case 'connecting':
        if (this.props.tunnelState.details) {
          pq = this.props.tunnelState.details.endpoint.quantumResistant;
        }
        break;
      case 'connected':
        pq = this.props.tunnelState.details.endpoint.quantumResistant;
        break;
    }

    switch (state) {
      case 'connecting': {
        const displayStyle = pq ? SecuredDisplayStyle.securingPq : SecuredDisplayStyle.securing;
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={displayStyle} />
              <Location>
                {this.renderCountry()}
                {this.renderCity()}
              </Location>
              <ConnectionPanelContainer />
            </Body>
            <Footer>
              <AppButton.ButtonGroup>
                {this.switchLocationButton()}
                <MultiButton mainButton={this.cancelButton} sideButton={this.reconnectButton} />
              </AppButton.ButtonGroup>
            </Footer>
          </Wrapper>
        );
      }

      case 'connected': {
        const displayStyle = pq ? SecuredDisplayStyle.securedPq : SecuredDisplayStyle.secured;
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={displayStyle} />
              <Location>
                {this.renderCountry()}
                {this.renderCity()}
              </Location>
              <ConnectionPanelContainer />
            </Body>
            <Footer>
              <AppButton.ButtonGroup>
                {this.switchLocationButton()}
                <MultiButton mainButton={this.disconnectButton} sideButton={this.reconnectButton} />
              </AppButton.ButtonGroup>
            </Footer>
          </Wrapper>
        );
      }

      case 'error':
        if (
          this.props.tunnelState.state === 'error' &&
          this.props.tunnelState.details.blockingError
        ) {
          return (
            <Wrapper>
              <Body>
                <Secured displayStyle={SecuredDisplayStyle.failedToSecure} />
              </Body>
              <Footer>
                <AppButton.ButtonGroup>
                  {this.switchLocationButton()}
                  <MultiButton mainButton={this.dismissButton} sideButton={this.reconnectButton} />
                </AppButton.ButtonGroup>
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
                <AppButton.ButtonGroup>
                  {this.switchLocationButton()}
                  <MultiButton mainButton={this.cancelButton} sideButton={this.reconnectButton} />
                </AppButton.ButtonGroup>
              </Footer>
            </Wrapper>
          );
        }

      case 'disconnecting':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.unsecuring} />
              <Location>
                {this.renderCountry()}
                <LocationRow />
              </Location>
            </Body>
            <Footer>
              <AppButton.ButtonGroup>
                {this.selectLocationButton()}
                {this.connectButton()}
              </AppButton.ButtonGroup>
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
              <Location>
                {this.renderCountry()}
                <LocationRow />
              </Location>
            </Body>
            <Footer>
              <AppButton.ButtonGroup>
                {this.selectLocationButton()}
                {this.connectButton()}
              </AppButton.ButtonGroup>
            </Footer>
          </Wrapper>
        );
      }

      default:
        throw new Error(`Unknown TunnelState: ${this.props.tunnelState}`);
    }
  }

  private renderCity() {
    const city = this.props.city === undefined ? '' : relayLocations.gettext(this.props.city);
    return (
      <LocationRow>
        <StyledMarquee data-testid="city">{city}</StyledMarquee>
      </LocationRow>
    );
  }

  private renderCountry() {
    const country =
      this.props.country === undefined ? '' : relayLocations.gettext(this.props.country);
    return (
      <LocationRow>
        <StyledMarquee data-testid="country">{country}</StyledMarquee>
      </LocationRow>
    );
  }

  private switchLocationButton() {
    return (
      <AppButton.TransparentButton onClick={this.props.onSelectLocation}>
        {messages.pgettext('tunnel-control', 'Switch location')}
      </AppButton.TransparentButton>
    );
  }

  private selectLocationButton() {
    return (
      <AppButton.TransparentButton
        onClick={this.props.onSelectLocation}
        aria-label={sprintf(
          messages.pgettext('accessibility', 'Select location. Current location is %(location)s'),
          { location: this.props.selectedRelayName },
        )}>
        <AppButton.Label>{this.props.selectedRelayName}</AppButton.Label>
        <SelectedLocationChevron height={12} width={7} source="icon-chevron" />
      </AppButton.TransparentButton>
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
        <AppButton.Label>
          <ImageView height={22} width={22} source="icon-reload" tintColor="white" />
        </AppButton.Label>
      </AppButton.RedTransparentButton>
    );
  };
}
