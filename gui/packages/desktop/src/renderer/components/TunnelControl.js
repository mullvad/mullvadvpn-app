// @flow

import * as React from 'react';
import { Component, Text, View, Styles, Types } from 'reactxp';
import { ConnectionInfo, SecuredLabel, SecuredDisplayStyle, ImageView } from '@mullvad/components';
import * as AppButton from './AppButton';
import { colors } from '../../config';

import type { TunnelStateTransition, RelayProtocol } from '../../shared/daemon-rpc-types';

export type RelayInAddress = {
  ip: string,
  port: number,
  protocol: RelayProtocol,
};

export type RelayOutAddress = {
  ipv4: ?string,
  ipv6: ?string,
};

type TunnelControlProps = {
  tunnelState: TunnelStateTransition,
  selectedRelayName: string,
  city: ?string,
  country: ?string,
  hostname: ?string,
  defaultConnectionInfoOpen?: boolean,
  relayInAddress: ?RelayInAddress,
  relayOutAddress: ?RelayOutAddress,
  onConnect: () => void,
  onDisconnect: () => void,
  onSelectLocation: () => void,
  onToggleConnectionInfo: (boolean) => void,
};

const styles = {
  body: Styles.createViewStyle({
    paddingTop: 0,
    paddingLeft: 24,
    paddingRight: 24,
    paddingBottom: 0,
    marginTop: 186,
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
  switch_location_button: Styles.createViewStyle({
    marginBottom: 16,
  }),
  status_security: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '800',
    lineHeight: 22,
    marginBottom: 4,
  }),
  status_location: Styles.createTextStyle({
    flexDirection: 'column',
    marginBottom: 4,
  }),
  status_location_text: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 34,
    lineHeight: 36,
    fontWeight: '900',
    overflow: 'hidden',
    letterSpacing: -0.9,
    color: colors.white,
  }),
};

export default class TunnelControl extends Component<TunnelControlProps> {
  render() {
    const Location = ({ children }) => <View style={styles.status_location}>{children}</View>;
    const City = () => <Text style={styles.status_location_text}>{this.props.city}</Text>;
    const Country = () => <Text style={styles.status_location_text}>{this.props.country}</Text>;

    const SwitchLocation = () => {
      return (
        <AppButton.TransparentButton
          style={styles.switch_location_button}
          onPress={this.props.onSelectLocation}>
          <AppButton.Label>{'Switch location'}</AppButton.Label>
        </AppButton.TransparentButton>
      );
    };

    const SelectedLocation = () => (
      <AppButton.TransparentButton
        style={styles.switch_location_button}
        onPress={this.props.onSelectLocation}>
        <AppButton.Label>{this.props.selectedRelayName}</AppButton.Label>
        <ImageView height={12} width={7} source="icon-chevron" />
      </AppButton.TransparentButton>
    );

    const Connect = () => (
      <AppButton.GreenButton onPress={this.props.onConnect}>
        {'Secure my connection'}
      </AppButton.GreenButton>
    );

    const Disconnect = () => (
      <AppButton.RedTransparentButton onPress={this.props.onDisconnect}>
        {'Disconnect'}
      </AppButton.RedTransparentButton>
    );

    const Cancel = () => (
      <AppButton.RedTransparentButton onPress={this.props.onDisconnect}>
        {'Cancel'}
      </AppButton.RedTransparentButton>
    );

    const Secured = ({ displayStyle }) => (
      <SecuredLabel style={styles.status_security} displayStyle={displayStyle} />
    );
    const Footer = ({ children }) => <View style={styles.footer}>{children}</View>;

    const connectionDetails = (
      <ConnectionInfo
        hostname={this.props.hostname}
        inAddress={this.props.relayInAddress}
        outAddress={this.props.relayOutAddress}
        defaultOpen={this.props.defaultConnectionInfoOpen}
        onToggle={this.props.onToggleConnectionInfo}
      />
    );

    let state = this.props.tunnelState.state;

    switch (state) {
      case 'blocked':
        switch (this.props.tunnelState.details.reason) {
          case 'set_security_policy_error':
            state = 'disconnected';
            break;
          default:
            state = 'blocked';
            break;
        }
        break;

      case 'disconnecting':
        switch (this.props.tunnelState.details) {
          case 'block':
            state = 'blocked';
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
              {connectionDetails}
            </Body>
            <Footer>
              <SwitchLocation />
              <Cancel />
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
              {connectionDetails}
            </Body>
            <Footer>
              <SwitchLocation />
              <Disconnect />
            </Footer>
          </Wrapper>
        );

      case 'blocked':
        return (
          <Wrapper>
            <Body>
              <Secured displayStyle={SecuredDisplayStyle.blocked} />
            </Body>
            <Footer>
              <SwitchLocation />
              <Cancel />
            </Footer>
          </Wrapper>
        );

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
        throw new Error(`Unknown TunnelState: ${(this.props.tunnelState: empty)}`);
    }
  }
}

type ContainerProps = {
  children?: Types.ReactNode,
};

class Wrapper extends Component<ContainerProps> {
  render() {
    return <View style={styles.wrapper}>{this.props.children}</View>;
  }
}

class Body extends Component<ContainerProps> {
  render() {
    return <View style={styles.body}>{this.props.children}</View>;
  }
}
