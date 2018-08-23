import * as React from 'react';
import { Animated, Component, Styles, Types, UserInterface, View } from 'reactxp';

interface IProps {
  height: number | 'auto';
  animationDuration?: number;
  style?: Types.AnimatedViewStyleRuleSet;
  children?: React.ReactNode;
}

interface IState {
  animatedValue: Animated.Value | null;
}

const containerOverflowStyle = Styles.createViewStyle({ overflow: 'hidden' });

export default class Accordion extends Component<IProps, IState> {
  public static defaultProps = {
    height: 'auto',
    animationDuration: 350,
  };

  public state: IState = {
    animatedValue: null,
  };

  private containerRef = React.createRef<Animated.View>();
  private contentHeight = 0;
  private animation: Types.Animated.CompositeAnimation | null = null;

  constructor(props: IProps) {
    super(props);

    // set the initial height if it's known
    if (typeof props.height === 'number') {
      this.state = {
        animatedValue: Animated.createValue(props.height),
      };
    }
  }

  public componentWillUnmount() {
    if (this.animation) {
      this.animation.stop();
    }
  }

  public shouldComponentUpdate(nextProps: IProps, nextState: IState) {
    return (
      nextState.animatedValue !== this.state.animatedValue ||
      nextProps.height !== this.props.height ||
      nextProps.children !== this.props.children
    );
  }

  public componentDidUpdate(prevProps: IProps, prevState: IState) {
    if (prevProps.height !== this.props.height) {
      this.animateHeightChanges();
    }
  }

  public render() {
    const { style, height, children, animationDuration, ...otherProps } = this.props;
    const containerStyles = [style];

    if (this.state.animatedValue !== null) {
      const animatedStyle = Styles.createAnimatedViewStyle({
        height: this.state.animatedValue,
      });

      containerStyles.push(containerOverflowStyle, animatedStyle);
    }

    return (
      <Animated.View
        {...otherProps}
        style={containerStyles}
        ref={
          /* Fix: cast to any because reactxp has out of date annotations
             See: https://github.com/Microsoft/reactxp/issues/784
           */
          this.containerRef as any
        }>
        <View onLayout={this.contentLayoutDidChange}>{children}</View>
      </Animated.View>
    );
  }

  private async animateHeightChanges() {
    const containerView = this.containerRef.current;
    if (!containerView) {
      return;
    }

    if (this.animation) {
      this.animation.stop();
      this.animation = null;
    }

    try {
      const layout = await UserInterface.measureLayoutRelativeToWindow(containerView);
      const fromValue = this.state.animatedValue || Animated.createValue(layout.height);
      const toValue = this.props.height === 'auto' ? this.contentHeight : this.props.height;

      // calculate the animation duration based on travel distance
      const multiplier = Math.abs(toValue - layout.height) / Math.max(1, this.contentHeight);
      const duration = Math.ceil(this.props.animationDuration! * multiplier);

      const animation = Animated.timing(fromValue, {
        toValue,
        easing: Animated.Easing.InOut(),
        duration,
        useNativeDriver: true,
      });

      this.animation = animation;
      this.setState({ animatedValue: fromValue }, () => {
        animation.start(this.onAnimationEnd);
      });
    } catch (error) {
      // TODO: log error
    }
  }

  private onAnimationEnd = ({ finished }: Types.Animated.EndResult) => {
    if (finished) {
      this.animation = null;

      // reset height after transition to let element layout naturally
      // if animation finished without interruption
      if (this.props.height === 'auto') {
        this.setState({ animatedValue: null });
      }
    }
  };

  private contentLayoutDidChange = ({ height }: Types.ViewOnLayoutEvent) =>
    (this.contentHeight = height);
}
