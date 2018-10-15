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
  inAddress: IInAddress;
  outAddress: IOutAddress;
  startExpanded: boolean;
  onToggle: (expanded: boolean) => void;
}

interface IState {
  expanded: boolean;
}

export default class ConnectionInfo extends Component<IProps, IState> {
  public static defaultProps = {
    inAddress: {
      ip: null,
      port: null,
      protocol: null,
    },
    outAddress: null,
    startExpanded: false,
    onToggle: (_: boolean) => {},
  };

  constructor(props: IProps) {
    super(props);

    this.state = {
      expanded: props.startExpanded,
    };
  }

  public render() {
    return (
      <View>
        <Accordion height={this.state.expanded ? 'auto' : 0}>
          <Text style={styles.content}>{this.inAddress()}</Text>
          <Text style={styles.content}>{this.outAddress()}</Text>
        </Accordion>
        <Text style={styles.toggle} onPress={() => this.toggle()}>
          {this.state.expanded ? 'LESS' : 'MORE'}
        </Text>
      </View>
    );
  }

  private toggle() {
    this.setState((state, props) => {
      const expanded = !state.expanded;

      props.onToggle(expanded);

      return { expanded };
    });
  }

  private inAddress() {
    const { ip, port, protocol } = this.props.inAddress;

    return (
      'IN: ' + (ip || '<unknown>') + (port ? `:${port}` : '') + (protocol ? ` - ${protocol}` : '')
    );
  }

  private outAddress() {
    const { ipv4, ipv6 } = this.props.outAddress;

    if (ipv4 || ipv6) {
      return `OUT: ${ipv4}` + (ipv6 ? ` / ${ipv6}` : '');
    } else {
      return '';
    }
  }
}
