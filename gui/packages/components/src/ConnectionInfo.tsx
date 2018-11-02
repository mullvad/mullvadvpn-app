import * as React from 'react';
import { Component, Styles, Text, View } from 'reactxp';
import { default as TriangleDisclosure } from './TriangleDisclosure';

const styles = {
  row: Styles.createViewStyle({
    flexDirection: 'row',
    marginTop: 3,
    marginLeft: 11,
  }),
  caption: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    color: 'rgb(255, 255, 255)',
    flex: 0,
    flexBasis: 30,
    marginRight: 8,
  }),
  value: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    color: 'rgb(255, 255, 255)',
    letterSpacing: -0.2,
  }),
  disclosure: Styles.createViewStyle({
    alignItems: 'flex-start',
  }),
};

interface IInAddress {
  ip: string;
  port: number;
  protocol: string;
}

interface IOutAddress {
  ipv4?: string;
  ipv6?: string;
}

interface IProps {
  inAddress?: IInAddress;
  outAddress?: IOutAddress;
  defaultOpen?: boolean;
  onToggle?: (isOpen: boolean) => void;
}

interface IState {
  isOpen: boolean;
}

export default class ConnectionInfo extends Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    this.state = {
      isOpen: !!props.defaultOpen,
    };
  }

  public render() {
    const { inAddress, outAddress } = this.props;

    return (
      <View>
        <View style={styles.disclosure}>
          <TriangleDisclosure defaultOpen={this.props.defaultOpen} onToggle={this.onToggle}>
            {'Connection details'}
          </TriangleDisclosure>
        </View>

        {this.state.isOpen && (
          <React.Fragment>
            {inAddress && (
              <View style={styles.row}>
                <Text style={styles.caption}>{'In'}</Text>
                <Text style={styles.value}>
                  {`${inAddress.ip}:${inAddress.port} ${inAddress.protocol.toUpperCase()}`}
                </Text>
              </View>
            )}

            {outAddress &&
              (outAddress.ipv4 || outAddress.ipv6) && (
                <View style={styles.row}>
                  <Text style={styles.caption}>{'Out'}</Text>
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

  private onToggle = (isOpen: boolean) => {
    this.setState(
      (state) => ({ ...state, isOpen }),
      () => {
        if (this.props.onToggle) {
          this.props.onToggle(isOpen);
        }
      },
    );
  };
}
