import log from 'electron-log';
import * as React from 'react';
import { Button, Component, Styles, Text, Types, UserInterface, View } from 'reactxp';
import { colors } from '../../config.json';
import consumePromise from '../../shared/promise';
import styles from './AppButtonStyles';
import ImageView from './ImageView';

const ButtonContext = React.createContext({
  textAdjustment: 0,
  textRef: React.createRef<PrivateLabel>(),
});

interface ILabelProps {
  children?: React.ReactText;
}

interface IPrivateLabelProps {
  textAdjustment: number;
  children?: React.ReactText;
}

class PrivateLabel extends Component<IPrivateLabelProps> {
  public render() {
    const { textAdjustment, children } = this.props;
    const textAdjustmentStyle = Styles.createViewStyle(
      {
        paddingRight: textAdjustment > 0 ? textAdjustment : 0,
        paddingLeft: textAdjustment < 0 ? Math.abs(textAdjustment) : 0,
      },
      false,
    );

    return (
      <View style={[styles.labelContainer, textAdjustmentStyle]}>
        <Text style={styles.label}>{children}</Text>
      </View>
    );
  }
}

export class Label extends Component<ILabelProps> {
  public render() {
    return (
      <ButtonContext.Consumer>
        {(context) => (
          <PrivateLabel ref={context.textRef} textAdjustment={context.textAdjustment}>
            {this.props.children}
          </PrivateLabel>
        )}
      </ButtonContext.Consumer>
    );
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
      />
    );
  }
}

interface IProps {
  children?: React.ReactNode;
  style?: Types.StyleRuleSetRecursive<Types.ButtonStyleRuleSet>;
  disabled?: boolean;
  onPress?: () => void;
  textOffset?: number;
}

interface IState {
  hovered: boolean;
  textAdjustment: number;
}

class BaseButton extends Component<IProps, IState> {
  public state: IState = {
    hovered: false,
    textAdjustment: 0,
  };

  private containerRef = React.createRef<View>();
  private textViewRef = React.createRef<PrivateLabel>();

  public componentDidMount() {
    consumePromise(this.forceUpdateTextAdjustment());
  }

  public render() {
    const { children, style, ...otherProps } = this.props;

    return (
      <ButtonContext.Provider
        value={{
          textAdjustment: this.state.textAdjustment,
          textRef: this.textViewRef,
        }}>
        <Button
          {...otherProps}
          style={[styles.common, this.backgroundStyle(), style]}
          onHoverStart={this.onHoverStart}
          onHoverEnd={this.onHoverEnd}>
          <View style={styles.content} ref={this.containerRef} onLayout={this.onLayout}>
            {React.Children.map(children, (child) =>
              typeof child === 'string' ? <Label>{child as string}</Label> : child,
            )}
          </View>
        </Button>
      </ButtonContext.Provider>
    );
  }

  protected backgroundStyle = (): Types.ButtonStyleRuleSet => {
    throw new Error('Implement backgroundStyle in subclasses.');
  };
  protected onHoverStart = () => (!this.props.disabled ? this.setState({ hovered: true }) : null);
  protected onHoverEnd = () => (!this.props.disabled ? this.setState({ hovered: false }) : null);

  private async forceUpdateTextAdjustment() {
    const containerView = this.containerRef.current;
    if (containerView) {
      const containerLayout = await UserInterface.measureLayoutRelativeToAncestor(
        containerView,
        this,
      );

      await this.updateTextAdjustment(containerLayout);
    }
  }

  private async updateTextAdjustment(containerLayout: Types.LayoutInfo) {
    const textOffset = this.props.textOffset || 0;
    const labelView = this.textViewRef.current;

    if (labelView) {
      // calculate the title layout frame
      const labelLayout = await UserInterface.measureLayoutRelativeToAncestor(labelView, this);

      // calculate the remaining space at the right hand side
      const trailingSpace = containerLayout.width - (labelLayout.x + labelLayout.width);

      // calculate text adjustment
      const textAdjustment = labelLayout.x - trailingSpace - textOffset;

      // re-render the view with the new text adjustment if it changed
      if (this.state.textAdjustment !== textAdjustment) {
        this.setState({ textAdjustment });
      }
    }
  }

  private onLayout = (containerLayout: Types.ViewOnLayoutEvent) => {
    consumePromise(this.updateTextAdjustment(containerLayout));
  };
}

interface IBlockingState {
  isBlocked: boolean;
}

interface IBlockingProps {
  children?: React.ReactNode;
  onPress: () => Promise<void>;
  disabled?: boolean;
}

export class BlockingButton extends Component<IBlockingProps, IBlockingState> {
  public state = {
    isBlocked: false,
  };

  public render() {
    return React.Children.map(this.props.children, (child) => {
      if (React.isValidElement(child)) {
        return React.cloneElement(child as React.ReactElement, {
          ...child.props,
          disabled: this.state.isBlocked || this.props.disabled,
          onPress: this.onPress,
        });
      } else {
        return child;
      }
    });
  }

  private onPress = () => {
    this.setState({ isBlocked: true }, async () => {
      try {
        await this.props.onPress();
      } catch (error) {
        log.error(`onPress() failed - ${error}`);
      }
      this.setState({ isBlocked: false });
    });
  };
}

export class RedButton extends BaseButton {
  protected backgroundStyle = () => (this.state.hovered ? styles.redHover : styles.red);
}

export class GreenButton extends BaseButton {
  protected backgroundStyle = () => (this.state.hovered ? styles.greenHover : styles.green);
}

export class BlueButton extends BaseButton {
  protected backgroundStyle = () => (this.state.hovered ? styles.blueHover : styles.blue);
}

export class TransparentButton extends BaseButton {
  protected backgroundStyle = () =>
    this.state.hovered ? styles.transparentHover : styles.transparent;
}

export class RedTransparentButton extends BaseButton {
  protected backgroundStyle = () =>
    this.state.hovered ? styles.redTransparentHover : styles.redTransparent;
}
