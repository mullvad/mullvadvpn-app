import * as React from 'react';
import { Animated, Component, Styles, Types, UserInterface, View } from 'reactxp';

interface IProps {
  expanded: boolean;
  animationDuration: number;
  style?: Types.AnimatedViewStyleRuleSet;
  children?: React.ReactNode;
}

interface IState {
  applyAnimatedStyle: boolean;
  mountChildren: boolean;
}

const containerOverflowStyle = Styles.createViewStyle({ overflow: 'hidden' });

export default class Accordion extends Component<IProps, IState> {
  public static defaultProps = {
    expanded: true,
    animationDuration: 350,
  };

  public state: IState = {
    applyAnimatedStyle: false,
    mountChildren: false,
  };

  private heightValue = Animated.createValue(0);
  private animatedStyle = Styles.createAnimatedViewStyle({
    height: this.heightValue,
  });

  private containerRef = React.createRef<Animated.View>();
  private contentRef = React.createRef<View>();
  private animation?: Types.Animated.CompositeAnimation = undefined;

  constructor(props: IProps) {
    super(props);

    this.state = {
      applyAnimatedStyle: !props.expanded,
      mountChildren: props.expanded,
    };
  }

  public componentWillUnmount() {
    if (this.animation) {
      this.animation.stop();
    }
  }

  public componentDidUpdate(oldProps: IProps, oldState: IState) {
    if (this.props.expanded !== oldProps.expanded) {
      // make sure the children are mounted first before expanding the accordion
      if (this.props.expanded && !this.state.mountChildren) {
        this.setState({ mountChildren: true });
      } else {
        this.animate(this.props.expanded);
      }
    } else if (this.state.mountChildren && !oldState.mountChildren) {
      // run animations once the children are mounted
      this.animate(this.props.expanded);
    }
  }

  public render() {
    const { style, children, expanded, animationDuration, ...otherProps } = this.props;
    const containerStyles = this.state.applyAnimatedStyle
      ? [style, containerOverflowStyle, this.animatedStyle]
      : [style];

    return (
      <Animated.View {...otherProps} style={containerStyles} ref={this.containerRef}>
        <View ref={this.contentRef}>{this.state.mountChildren && children}</View>
      </Animated.View>
    );
  }

  private async animate(expand: boolean) {
    const containerView = this.containerRef.current;
    const contentView = this.contentRef.current;
    if (!containerView || !contentView) {
      return;
    }

    if (this.animation) {
      this.animation.stop();
      this.animation = undefined;
    }

    const containerLayout = await UserInterface.measureLayoutRelativeToWindow(containerView);
    const contentLayout = await UserInterface.measureLayoutRelativeToAncestor(
      contentView,
      containerView,
    );

    // the content is expanded when the animated style is not applied,
    // so reset the initial animated value to the current layout's height.
    if (!this.state.applyAnimatedStyle) {
      this.heightValue.setValue(containerLayout.height);
    }

    const toValue = expand ? contentLayout.height : 0;

    // calculate the animation duration based on travel distance
    const multiplier =
      Math.abs(toValue - containerLayout.height) / Math.max(1, contentLayout.height);
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
        if (expand) {
          this.setState({ applyAnimatedStyle: false });
        }
      }
    };

    this.setState({ applyAnimatedStyle: true }, () => {
      animation.start(onAnimationEnd);
    });
  }
}
