import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import ConnectionPanelContainer from '../containers/ConnectionPanelContainer';
import * as AppButton from './AppButton';
import ImageView from './ImageView';
import Marquee from './Marquee';
import { MultiButton } from './MultiButton';
import SecuredLabel, { SecuredDisplayStyle } from './SecuredLabel';

interface ITunnelControlProps {
  tunnelState: TunnelState;
  selectedRelayName: string;
  city?: string;
  country?: string;
  onConnect: () => void;
  onDisconnect: () => void;
  onReconnect: () => void;
  onSelectLocation: () => void;
}

const SwitchLocationButton = styled(AppButton.TransparentButton)({
  marginBottom: 16,
});

const styles = {
  body: Styles.createViewStyle({
    paddingTop: 0,
    paddingLeft: 24,
    paddingRight: 24,
    paddingBottom: 0,
    marginTop: 176,
    flex: 1,
  }),
  footer: Styles.createViewStyle({
    flex: 0,
    paddingBottom: 16,
    paddingLeft: 24,
    paddingRight: 24,
  }),
  wrapper: Styles.createViewStyle({
    flex: 1,
  }),
  status_security: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '800',
    lineHeight: 22,
    marginBottom: 2,
  }),
  status_location: Styles.createTextStyle({
    flexDirection: 'column',
    marginBottom: 2,
  }),
};

const StyledMarquee = styled(Marquee)({
  fontFamily: 'DINPro',
  fontSize: '34px',
  lineHeight: '38px',
  fontWeight: 900,
  overflow: 'hidden',
  letterSpacing: -0.9,
  color: colors.white,
});

export default class TunnelControl extends Component<ITunnelControlProps> {
  public render() {
    const Location = ({ children }: { children?: React.ReactNode }) => (
      <View style={styles.status_location}>{children}</View>
    );
    const City = () => <StyledMarquee>{this.props.city}</StyledMarquee>;
    const Country = () => <StyledMarquee>{this.props.country}</StyledMarquee>;

    const SwitchLocation = () => {
      return (
        <SwitchLocationButton onClick={this.props.onSelectLocation}>
          {messages.pgettext('tunnel-control', 'Switch location')}
        </SwitchLocationButton>
      );
    };

    const SelectedLocation = () => (
      <SwitchLocationButton onClick={this.props.onSelectLocation}>
        <AppButton.Label>{this.props.selectedRelayName}</AppButton.Label>
        <AppButton.Icon height={12} width={7} source="icon-chevron" />
      </SwitchLocationButton>
    );

    const Connect = () => (
      <AppButton.GreenButton onClick={this.props.onConnect}>
        {messages.pgettext('tunnel-control', 'Secure my connection')}
      </AppButton.GreenButton>
    );

    const Disconnect = (props: React.ComponentProps<typeof AppButton.RedTransparentButton>) => (
      <AppButton.RedTransparentButton onClick={this.props.onDisconnect} {...props}>
        {messages.pgettext('tunnel-control', 'Disconnect')}
      </AppButton.RedTransparentButton>
    );

    const Cancel = (props: React.ComponentProps<typeof AppButton.RedTransparentButton>) => (
      <AppButton.RedTransparentButton onClick={this.props.onDisconnect} {...props}>
        {messages.pgettext('tunnel-control', 'Cancel')}
      </AppButton.RedTransparentButton>
    );

    const Dismiss = (props: React.ComponentProps<typeof AppButton.RedTransparentButton>) => (
      <AppButton.RedTransparentButton onClick={this.props.onDisconnect} {...props}>
        {messages.pgettext('tunnel-control', 'Dismiss')}
      </AppButton.RedTransparentButton>
    );

    const Reconnect = (props: React.ComponentProps<typeof AppButton.RedTransparentButton>) => (
      <AppButton.RedTransparentButton onClick={this.props.onReconnect} {...props}>
        <ImageView height={22} width={22} source="icon-reload" tintColor="white" />
      </AppButton.RedTransparentButton>
    );

    const Secured = ({ displayStyle }: { displayStyle: SecuredDisplayStyle }) => (
      <SecuredLabel style={styles.status_security} displayStyle={displayStyle} />
    );
    const Footer = ({ children }: { children: React.ReactNode }) => (
      <View style={styles.footer}>{children}</View>
    );

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
                <City />
                <Country />
              </Location>
              <ConnectionPanelContainer />
            </Body>
            <Footer>
              <SwitchLocation />
              <MultiButton mainButton={Cancel} sideButton={Reconnect} />
            </Footer>
          </Wrapper>
        );
      case 'connected':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.secured} />
              <Location>
                <City />
                <Country />
              </Location>
              <ConnectionPanelContainer />
            </Body>
            <Footer>
              <SwitchLocation />
              <MultiButton mainButton={Disconnect} sideButton={Reconnect} />
            </Footer>
          </Wrapper>
        );

      case 'error':
        if (
          this.props.tunnelState.state === 'error' &&
          !this.props.tunnelState.details.isBlocking
        ) {
          return (
            <Wrapper>
              <Body>
                <Secured displayStyle={SecuredDisplayStyle.failedToSecure} />
              </Body>
              <Footer>
                <SwitchLocation />
                <MultiButton mainButton={Dismiss} sideButton={Reconnect} />
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
                <SwitchLocation />
                <MultiButton mainButton={Cancel} sideButton={Reconnect} />
              </Footer>
            </Wrapper>
          );
        }

      case 'disconnecting':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.secured} />
              <Location>
                <Country />
              </Location>
            </Body>
            <Footer>
              <SelectedLocation />
              <Connect />
            </Footer>
          </Wrapper>
        );

      case 'disconnected':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.unsecured} />
              <Location>
                <Country />
              </Location>
            </Body>
            <Footer>
              <SelectedLocation />
              <Connect />
            </Footer>
          </Wrapper>
        );

      default:
        throw new Error(`Unknown TunnelState: ${this.props.tunnelState}`);
    }
  }
}

interface IContainerProps {
  children?: Types.ReactNode;
}

class Wrapper extends Component<IContainerProps> {
  public render() {
    return <View style={styles.wrapper}>{this.props.children}</View>;
  }
}

class Body extends Component<IContainerProps> {
  public render() {
    return <View style={styles.body}>{this.props.children}</View>;
  }
}
