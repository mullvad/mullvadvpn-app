import * as React from 'react';
import { Clipboard, Component, Text, Types } from 'reactxp';

interface IProps {
  value: string;
  delay: number;
  message: string;
  style?: Types.TextStyleRuleSet;
}

interface IState {
  showsMessage: boolean;
}

export default class ClipboardLabel extends Component<IProps, IState> {
  public static defaultProps: Partial<IProps> = {
    delay: 3000,
    message: 'Copied!',
  };

  public state: IState = {
    showsMessage: false,
  };

  private timer?: NodeJS.Timeout;

  public componentWillUnmount() {
    if (this.timer) {
      clearTimeout(this.timer);
    }
  }

  public render() {
    return (
      <Text style={this.props.style} onPress={this.handlePress}>
        {this.state.showsMessage ? this.props.message : this.props.value}
      </Text>
    );
  }

  private handlePress = () => {
    if (this.timer) {
      clearTimeout(this.timer);
    }

    this.timer = setTimeout(() => this.setState({ showsMessage: false }), this.props.delay);
    this.setState({ showsMessage: true });

    Clipboard.setText(this.props.value);
  };
}
