import * as React from 'react';
import { Component, Styles, Text, View } from 'reactxp';
import { default as Accordion } from './Accordion';

const styles = {
  toggle: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 14,
    fontWeight: '800',
    color: 'rgb(255, 255, 255, 0.4)',
    paddingBottom: 2,
  }),
  content: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '800',
    color: 'rgb(255, 255, 255)',
    paddingBottom: 2,
  }),
};

interface IInAddress {
  ip: string;
  port: number;
  protocol: string;
}

interface IOutAddress {
  ipv4: string | null;
  ipv6: string | null;
}

interface IProps {
  inAddress?: IInAddress;
  outAddress: IOutAddress;
  isExpanded: boolean;
  onToggle?: () => void;
}

export default class ConnectionInfo extends Component<IProps> {
  public render() {
    const { inAddress, outAddress } = this.props;

    return (
      <View>
        <Accordion height={this.props.isExpanded ? 'auto' : 0}>
          {inAddress && (
            <Text style={styles.content}>{`IN: ${inAddress.ip}:${inAddress.port} - ${
              inAddress.protocol
            }`}</Text>
          )}

          {(outAddress.ipv4 || outAddress.ipv6) && (
            <Text style={styles.content}>
              {'OUT: ' +
                [outAddress.ipv4, outAddress.ipv6]
                  .filter((a) => typeof a !== 'undefined')
                  .join(' / ')}
            </Text>
          )}
        </Accordion>
        <Text style={styles.toggle} onPress={this.toggle}>
          {this.props.isExpanded ? 'LESS' : 'MORE'}
        </Text>
      </View>
    );
  }

  private toggle = () => {
    if (this.props.onToggle) {
      this.props.onToggle();
    }
  };
}
