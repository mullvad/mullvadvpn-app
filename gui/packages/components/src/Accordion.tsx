import * as React from 'react';
import { Animated, Component, Styles, Types, UserInterface, View } from 'reactxp';
import AccordionContent from './AccordionContent';

interface IProps {
  defaultCollapsed: boolean;
  animationDuration: number;
  style?: Types.AnimatedViewStyleRuleSet;
  children?: React.ReactNode;
}

interface IState {
  applyAnimatedStyle: boolean;
}

const containerOverflowStyle = Styles.createViewStyle({ overflow: 'hidden' });

export default class Accordion extends Component<IProps, IState> {
  public static defaultProps = {
    defaultCollapsed: false,
    animationDuration: 350,
  };

  public state: IState = {
    applyAnimatedStyle: false,
  };

  private heightValue = Animated.createValue(0);
  private animatedStyle = Styles.createAnimatedViewStyle({
    height: this.heightValue,
  });

  private collapsed = false;
  private containerRef = React.createRef<Animated.View>();
  private contentHeight = 0;
  private animation?: Types.Animated.CompositeAnimation = undefined;
  private contentCacheKey = 0;

  constructor(props: IProps) {
    super(props);

    this.collapsed = props.defaultCollapsed;

    if (props.defaultCollapsed) {
      this.state.applyAnimatedStyle = true;
    }
  }

  public UNSAFE_componentWillReceiveProps(nextProps: IProps) {
    // bump the content cache key on prop changes to force the accordion contents to be updated.
    this.contentCacheKey += 1;
  }

  public componentWillUnmount() {
    if (this.animation) {
      this.animation.stop();
    }
  }

  public render() {
    const { style, children, defaultCollapsed, animationDuration, ...otherProps } = this.props;
    const containerStyles = [style];

    if (this.state.applyAnimatedStyle) {
      containerStyles.push(containerOverflowStyle, this.animatedStyle);
    }

    return (
      <Animated.View {...otherProps} style={containerStyles} ref={this.containerRef}>
        <AccordionContent cacheKey={this.contentCacheKey}>
          <View onLayout={this.contentLayoutDidChange}>{children}</View>
        </AccordionContent>
      </Animated.View>
    );
  }

  public get isCollapsed(): boolean {
    return this.collapsed;
  }

  public collapse() {
    this.collapsed = true;
    this.animate(true);
  }

  public expand() {
    this.collapsed = false;
    this.animate(false);
  }

  public toggle() {
    const collapsed = !this.collapsed;
    this.collapsed = collapsed;
    this.animate(collapsed);
  }

  private async animate(collapse: boolean) {
    const containerView = this.containerRef.current;
    if (!containerView) {
      return;
    }

    if (this.animation) {
      this.animation.stop();
      this.animation = undefined;
    }

    let layout: Types.LayoutInfo;
    try {
      layout = await UserInterface.measureLayoutRelativeToWindow(containerView);
    } catch (error) {
      // TODO: log error
      return;
    }

    // the content is expanded when the animated style is not applied,
    // so reset the initial animated value to the current layout's height.
    if (!this.state.applyAnimatedStyle) {
      this.heightValue.setValue(layout.height);
    }

    const toValue = collapse ? 0 : this.contentHeight;

    // calculate the animation duration based on travel distance
    const multiplier = Math.abs(toValue - layout.height) / Math.max(1, this.contentHeight);
    const duration = Math.ceil(this.props.animationDuration * multiplier);

    const animation = Animated.timing(this.heightValue, {
      toValue,
      easing: Animated.Easing.InOut(),
      duration,
      useNativeDriver: true,
    });

    this.animation = animation;

    const onAnimationEnd = ({ finished }: Types.Animated.EndResult) => {
      if (finished) {
        this.animation = undefined;

        // reset the height after transition to let element layout naturally
        // if animation finished without interruption
        if (!collapse) {
          this.setState({ applyAnimatedStyle: false });
        }
      }
    };

    this.setState({ applyAnimatedStyle: true }, () => {
      animation.start(onAnimationEnd);
    });
  }

  private contentLayoutDidChange = ({ height }: Types.ViewOnLayoutEvent) =>
    (this.contentHeight = height);
}
