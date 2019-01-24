import * as React from 'react';
import { Styles, Component, Animated, View, Types, UserInterface } from 'reactxp';
import { TransitionGroupProps } from '../transitions';

type Props = {
  children: React.ReactNode;
} & TransitionGroupProps;

type State = {
  previousChildren?: React.ReactNode;
  childrenAnimation?: Types.AnimatedViewStyleRuleSet;
  previousChildrenAnimation?: Types.AnimatedViewStyleRuleSet;
  dimensions: Types.Dimensions;
};

const dimensions = UserInterface.measureWindow();
const styles = {
  animationDefaultStyle: Styles.createViewStyle({
    // @ts-ignore
    position: 'absolute',
    width: dimensions.width,
    height: dimensions.height,
  }),
  allowPointerEventsStyle: Styles.createViewStyle({
    // @ts-ignore
    pointerEvents: 'auto',
  }),
  transitionContainerStyle: Styles.createViewStyle({
    width: dimensions.width,
    height: dimensions.height,
  }),
};

export default class TransitionContainer extends Component<Props, State> {
  constructor(props: Props) {
    super(props);

    this.state = {
      dimensions: UserInterface.measureWindow(),
    };
  }

  UNSAFE_componentWillReceiveProps(nextProps: Props) {
    switch (nextProps.name) {
      case 'slide-up':
        this.slideUpTransition(nextProps);
        break;
      case 'slide-down':
        this.slideDownTransition(nextProps);
        break;
      case 'push':
        this.pushTransition(nextProps);
        break;
      case 'pop':
        this.popTransition(nextProps);
        break;
      default:
        break;
    }
  }

  onFinishedAnimation() {
    this.setState({
      childrenAnimation: styles.allowPointerEventsStyle,
      previousChildren: null,
    });
  }

  slideUpTransition(nextProps: Props) {
    const currentTranslationValue = Animated.createValue(this.state.dimensions.height);
    this.setState(
      {
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateY: currentTranslationValue }],
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{ translateY: Animated.createValue(0) }],
        }),
      },
      () => {
        Animated.timing(currentTranslationValue, {
          toValue: 0,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
        }).start(() => this.onFinishedAnimation());
      },
    );
  }

  slideDownTransition(nextProps: Props) {
    const previousTranslationValue = Animated.createValue(0);
    this.setState(
      {
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{ translateY: Animated.createValue(0) }],
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateY: previousTranslationValue }],
        }),
      },
      () => {
        Animated.timing(previousTranslationValue, {
          toValue: this.state.dimensions.height,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
        }).start(() => this.onFinishedAnimation());
      },
    );
  }

  pushTransition(nextProps: Props) {
    const currentTranslationValue = Animated.createValue(this.state.dimensions.width);
    const previousTranslationValue = Animated.createValue(0);
    this.setState(
      {
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateX: currentTranslationValue }],
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{ translateX: previousTranslationValue }],
        }),
      },
      () => {
        const compositeAnimation = Animated.parallel([
          Animated.timing(currentTranslationValue, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
          }),
          Animated.timing(previousTranslationValue, {
            toValue: -this.state.dimensions.width / 2,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
          }),
        ]);
        compositeAnimation.start(() => this.onFinishedAnimation());
      },
    );
  }

  popTransition(nextProps: Props) {
    const currentTranslationValue = Animated.createValue(-this.state.dimensions.width / 2);
    const previousTranslationValue = Animated.createValue(0);
    this.setState(
      {
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{ translateX: currentTranslationValue }],
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          // @ts-ignore
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateX: previousTranslationValue }],
        }),
      },
      () => {
        const compositeAnimation = Animated.parallel([
          Animated.timing(currentTranslationValue, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
          }),
          Animated.timing(previousTranslationValue, {
            toValue: this.state.dimensions.width,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
          }),
        ]);
        compositeAnimation.start(() => this.onFinishedAnimation());
      },
    );
  }

  render() {
    const { children } = this.props;
    const { previousChildren, childrenAnimation, previousChildrenAnimation } = this.state;

    return (
      <View style={styles.transitionContainerStyle}>
        {previousChildren && (
          <Animated.View
            key={getChildKey(previousChildren)}
            style={[styles.animationDefaultStyle, previousChildrenAnimation]}>
            {previousChildren}
          </Animated.View>
        )}

        <Animated.View
          key={getChildKey(children)}
          style={[styles.animationDefaultStyle, childrenAnimation]}>
          {children}
        </Animated.View>
      </View>
    );
  }
}

function getChildKey(child?: React.ReactNode): string | number | undefined {
  return child &&
    typeof child === 'object' &&
    'key' in child &&
    (typeof child.key === 'string' || typeof child.key === 'number')
    ? child.key
    : undefined;
}
