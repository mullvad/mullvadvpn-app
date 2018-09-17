import * as React from 'react';
import { Clipboard, Component, Text, Types } from 'reactxp';

interface IProps {
  value: string;
  style: Types.TextStyleRuleSet;
}

interface IState {
  showsMessage: boolean;
}

export default class IpAddressLabel extends Component<IProps, IState> {
  private timer: NodeJS.Timer | null = null;

  public state: IState = {
    showsMessage: false,
  };

  public componentWillUnmount() {
    if (this.timer) {
      clearTimeout(this.timer);
    }
  }

  public render() {
    return (
      <Text style={this.props.style} onPress={this.handlePress}>
        {this.state.showsMessage ? 'IP copied to clipboard!' : this.props.value}
      </Text>
    );
  }

  private handlePress = () => {
    if (this.timer) {
      clearTimeout(this.timer);
    }
    this.timer = setTimeout(() => this.setState({ showsMessage: false }), 3000);
    this.setState({ showsMessage: true });
    Clipboard.setText(this.props.value);
  };
}
