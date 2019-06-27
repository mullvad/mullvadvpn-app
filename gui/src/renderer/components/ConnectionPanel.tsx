import * as React from 'react';
import { Component, Styles, Text, Types, View } from 'reactxp';
import { sprintf } from 'sprintf-js';
import {
  ProxyType,
  proxyTypeToString,
  RelayProtocol,
  TunnelType,
  tunnelTypeToString,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { default as ConnectionPanelDisclosure } from '../components/ConnectionPanelDisclosure';

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

export interface IOutAddress {
  ipv4?: string;
  ipv6?: string;
}

interface IProps {
  isOpen: boolean;
  hostname?: string;
  bridgeHostname?: string;
  inAddress?: IInAddress;
  bridgeInfo?: IBridgeData;
  outAddress?: IOutAddress;
  onToggle: () => void;
  style?: Types.ViewStyleRuleSet | Types.ViewStyleRuleSet[];
}

const styles = {
  row: Styles.createViewStyle({
    flexDirection: 'row',
    marginTop: 3,
  }),
  caption: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    color: 'rgb(255, 255, 255)',
    flex: 0,
    marginRight: 8,
  }),
  value: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    color: 'rgb(255, 255, 255)',
    letterSpacing: -0.2,
  }),
  header: Styles.createViewStyle({
    flexDirection: 'row',
    alignItems: 'center',
  }),
};

export default class ConnectionPanel extends Component<IProps> {
  public render() {
    const { inAddress, outAddress, bridgeInfo } = this.props;
    const entryPoint = bridgeInfo && inAddress ? bridgeInfo : inAddress;

    return (
      <View style={this.props.style}>
        {this.props.hostname && (
          <View style={styles.header}>
            <ConnectionPanelDisclosure pointsUp={this.props.isOpen} onToggle={this.props.onToggle}>
              {this.hostnameLine()}
            </ConnectionPanelDisclosure>
          </View>
        )}

        {this.props.isOpen && this.props.hostname && (
          <React.Fragment>
            {this.props.inAddress && (
              <View style={styles.row}>
                <Text style={styles.value}>{this.transportLine()}</Text>
              </View>
            )}

            {entryPoint && (
              <View style={styles.row}>
                <Text style={styles.caption}>{messages.pgettext('connection-info', 'In')}</Text>
                <Text style={styles.value}>
                  {`${entryPoint.ip}:${entryPoint.port} ${entryPoint.protocol.toUpperCase()}`}
                </Text>
              </View>
            )}

            {outAddress && (outAddress.ipv4 || outAddress.ipv6) && (
              <View style={styles.row}>
                <Text style={styles.caption}>{messages.pgettext('connection-info', 'Out')}</Text>
                <View>
                  {outAddress.ipv4 && <Text style={styles.value}>{outAddress.ipv4}</Text>}
                  {outAddress.ipv6 && <Text style={styles.value}>{outAddress.ipv6}</Text>}
                </View>
              </View>
            )}
          </React.Fragment>
        )}
      </View>
    );
  }

  private hostnameLine() {
    if (this.props.hostname && this.props.bridgeHostname) {
      return sprintf(
        // TRANSLATORS: The hostname line displayed below the country on the main screen
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(relay)s - the relay hostname
        // TRANSLATORS: %(bridge)s - the bridge hostname
        messages.pgettext('connection-info', '%(relay)s via %(bridge)s'),
        {
          relay: this.props.hostname,
          bridge: this.props.bridgeHostname,
        },
      );
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
