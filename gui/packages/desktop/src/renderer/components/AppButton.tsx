import * as React from 'react';
import { Button, Component, Text, Types } from 'reactxp';
import { colors } from '../../config.json';
import styles from './AppButtonStyles';
import ImageView from './ImageView';

interface ILabelProps {
  children?: React.ReactText;
}

export class Label extends Component<ILabelProps> {
  public render() {
    return <Text style={styles.label}>{this.props.children}</Text>;
  }
}

interface IIconProps {
  source: string;
  width?: number;
  height?: number;
}

export class Icon extends Component<IIconProps> {
  public render() {
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

interface IProps {
  children?: React.ReactNode;
  style?: Types.ButtonStyleRuleSet;
  disabled?: boolean;
  onPress?: () => void;
}

interface IState {
  hovered: boolean;
}

class BaseButton extends Component<IProps, IState> {
  public state: IState = { hovered: false };

  public backgroundStyle = (): Types.ButtonStyleRuleSet => {
    throw new Error('Implement backgroundStyle in subclasses.');
  };
  public onHoverStart = () => (!this.props.disabled ? this.setState({ hovered: true }) : null);
  public onHoverEnd = () => (!this.props.disabled ? this.setState({ hovered: false }) : null);

  public render() {
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
  public backgroundStyle = () => (this.state.hovered ? styles.redHover : styles.red);
}

export class GreenButton extends BaseButton {
  public backgroundStyle = () => (this.state.hovered ? styles.greenHover : styles.green);
}

export class BlueButton extends BaseButton {
  public backgroundStyle = () => (this.state.hovered ? styles.blueHover : styles.blue);
}

export class TransparentButton extends BaseButton {
  public backgroundStyle = () =>
    this.state.hovered ? styles.transparentHover : styles.transparent;
}

export class RedTransparentButton extends BaseButton {
  public backgroundStyle = () =>
    this.state.hovered ? styles.redTransparentHover : styles.redTransparent;
}
