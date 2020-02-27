import * as React from 'react';
import { Clipboard, Component, Text, Types } from 'reactxp';
import { messages } from '../../shared/gettext';

interface IProps {
  value: string;
  displayValue?: string;
  delay: number;
  message: string;
  style?: Types.StyleRuleSetRecursive<Types.TextStyleRuleSet>;
  messageStyle?: Types.StyleRuleSetRecursive<Types.TextStyleRuleSet>;
}

interface IState {
  showsMessage: boolean;
}

export default class ClipboardLabel extends Component<IProps, IState> {
  public static defaultProps: Partial<IProps> = {
    delay: 3000,
    message: messages.gettext('COPIED TO CLIPBOARD!'),
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
    const displayValue = this.props.displayValue || this.props.value;
    const style = this.state.showsMessage
      ? [this.props.style, this.props.messageStyle]
      : this.props.style;
    return (
      <Text style={style} onPress={this.handlePress}>
        {this.state.showsMessage ? this.props.message : displayValue}
      </Text>
    );
  }

  private handlePress = () => {
    if (this.timer) {
      clearTimeout(this.timer);
    }

    this.timer = global.setTimeout(() => this.setState({ showsMessage: false }), this.props.delay);
    this.setState({ showsMessage: true });

    Clipboard.setText(this.props.value);
  };
}
