import * as React from 'react';
import { Button, Component, Text, Types } from 'reactxp';
import { ImageView } from '@mullvad/components';
import styles from './AppButtonStyles';
import { colors } from '../../config.json';

type LabelProps = {
  children?: React.ReactText;
};

export class Label extends Component<LabelProps> {
  render() {
    return <Text style={styles.label}>{this.props.children}</Text>;
  }
}

type IconProps = {
  source: string;
  width?: number;
  height?: number;
};

export class Icon extends Component<IconProps> {
  render() {
    return (
      <ImageView
        source={this.props.source}
        width={this.props.width}
        height={this.props.height}
        tintColor={colors.white}
        style={styles.icon}
      />
    );
  }
}

type Props = {
  children?: React.ReactNode;
  style?: Types.ButtonStyleRuleSet;
  disabled?: boolean;
  onPress?: () => void;
};

type State = {
  hovered: boolean;
};

class BaseButton extends Component<Props, State> {
  state: State = { hovered: false };

  backgroundStyle = (): Types.ButtonStyleRuleSet => {
    throw new Error('Implement backgroundStyle in subclasses.');
  };
  onHoverStart = () => (!this.props.disabled ? this.setState({ hovered: true }) : null);
  onHoverEnd = () => (!this.props.disabled ? this.setState({ hovered: false }) : null);

  render() {
    const { children, style, ...otherProps } = this.props;

    return (
      <Button
        {...otherProps}
        style={[styles.common, this.backgroundStyle(), style]}
        onHoverStart={this.onHoverStart}
        onHoverEnd={this.onHoverEnd}>
        {React.Children.map(children, (child) =>
          typeof child === 'string' ? <Label>{child as string}</Label> : child,
        )}
      </Button>
    );
  }
}

export class RedButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.redHover : styles.red);
}

export class GreenButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.greenHover : styles.green);
}

export class BlueButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.blueHover : styles.blue);
}

export class TransparentButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.transparentHover : styles.transparent);
}

export class RedTransparentButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.redTransparentHover : styles.redTransparent);
}
