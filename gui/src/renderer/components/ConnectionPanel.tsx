import * as React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../config.json';
import {
  ObfuscationType,
  ProxyType,
  proxyTypeToString,
  RelayProtocol,
  TunnelType,
  tunnelTypeToString,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { default as ConnectionPanelDisclosure } from '../components/ConnectionPanelDisclosure';
import { tinyText } from './common-styles';
import Marquee from './Marquee';

export interface IEndpoint {
  ip: string;
  port: number;
  protocol: RelayProtocol;
}

export interface IInAddress extends IEndpoint {
  tunnelType: TunnelType;
}

export interface IBridgeData extends IEndpoint {
  bridgeType: ProxyType;
}

export interface IObfuscationData extends IEndpoint {
  obfuscationType: ObfuscationType;
}

export interface IOutAddress {
  ipv4?: string;
  ipv6?: string;
}

interface IProps {
  isOpen: boolean;
  hostname?: string;
  bridgeHostname?: string;
  entryHostname?: string;
  inAddress?: IInAddress;
  entryLocationInAddress?: IInAddress;
  bridgeInfo?: IBridgeData;
  outAddress?: IOutAddress;
  obfuscationEndpoint?: IObfuscationData;
  onToggle: () => void;
  className?: string;
}

const Container = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

const Row = styled.div({
  display: 'flex',
  marginTop: '3px',
});

const Text = styled.span(tinyText, {
  lineHeight: '15px',
  color: colors.white,
});

const Caption = styled(Text)({
  flex: 0,
  marginRight: '8px',
});

const IpAddresses = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

const Header = styled.div({
  alignSelf: 'start',
  display: 'flex',
  alignItems: 'center',
  width: '100%',
});

export default class ConnectionPanel extends React.Component<IProps> {
  public render() {
    const { outAddress } = this.props;
    const entryPoint = this.getEntryPoint();

    return (
      <Container className={this.props.className}>
        {this.props.hostname && (
          <Header>
            <ConnectionPanelDisclosure pointsUp={this.props.isOpen} onToggle={this.props.onToggle}>
              <Marquee>{this.hostnameLine()}</Marquee>
            </ConnectionPanelDisclosure>
          </Header>
        )}

        {this.props.isOpen && this.props.hostname && (
          <React.Fragment>
            {this.props.inAddress && (
              <Row>
                <Text>{this.transportLine()}</Text>
              </Row>
            )}

            {entryPoint && (
              <Row>
                <Caption>{messages.pgettext('connection-info', 'In')}</Caption>
                <Text>
                  {`${entryPoint.ip}:${entryPoint.port} ${entryPoint.protocol.toUpperCase()}`}
                </Text>
              </Row>
            )}

            {outAddress && (outAddress.ipv4 || outAddress.ipv6) && (
              <Row>
                <Caption>{messages.pgettext('connection-info', 'Out')}</Caption>
                <IpAddresses>
                  {outAddress.ipv4 && <Text>{outAddress.ipv4}</Text>}
                  {outAddress.ipv6 && <Text>{outAddress.ipv6}</Text>}
                </IpAddresses>
              </Row>
            )}
          </React.Fragment>
        )}
      </Container>
    );
  }

  private getEntryPoint(): IEndpoint | undefined {
    const { obfuscationEndpoint, inAddress, entryLocationInAddress, bridgeInfo } = this.props;

    if (obfuscationEndpoint) {
      return obfuscationEndpoint;
    } else if (entryLocationInAddress && inAddress) {
      return entryLocationInAddress;
    } else if (bridgeInfo && inAddress) {
      return bridgeInfo;
    } else {
      return inAddress;
    }
  }

  private hostnameLine() {
    if (this.props.hostname && this.props.bridgeHostname) {
      return sprintf(messages.pgettext('connection-info', '%(relay)s via %(entry)s'), {
        relay: this.props.hostname,
        entry: this.props.bridgeHostname,
      });
    } else if (this.props.hostname && this.props.entryHostname) {
      return sprintf(
        // TRANSLATORS: The hostname line displayed below the country on the main screen
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(relay)s - the relay hostname
        // TRANSLATORS: %(entry)s - the entry relay hostname
        messages.pgettext('connection-info', '%(relay)s via %(entry)s'),
        {
          relay: this.props.hostname,
          entry: this.props.entryHostname,
        },
      );
    } else if (this.props.bridgeInfo?.ip) {
      return sprintf(messages.pgettext('connection-info', '%(relay)s via %(entry)s'), {
        relay: this.props.hostname,
        entry: this.props.bridgeInfo.ip,
      });
    } else {
      return this.props.hostname || '';
    }
  }

  private transportLine() {
    const { inAddress, bridgeInfo } = this.props;

    if (inAddress) {
      const tunnelType = tunnelTypeToString(inAddress.tunnelType);

      if (bridgeInfo) {
        const bridgeType = proxyTypeToString(bridgeInfo.bridgeType);

        return sprintf(
          // TRANSLATORS: The tunnel type line displayed below the hostname line on the main screen
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(tunnelType)s - the tunnel type, i.e OpenVPN
          // TRANSLATORS: %(bridgeType)s - the bridge type, i.e Shadowsocks
          messages.pgettext('connection-info', '%(tunnelType)s via %(bridgeType)s'),
          {
            tunnelType,
            bridgeType,
          },
        );
      } else {
        return tunnelType;
      }
    } else {
      return '';
    }
  }
}
