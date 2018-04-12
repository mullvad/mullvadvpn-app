// @flow
import * as React from 'react';
import { Styles, Component, Animated, View, Types, UserInterface } from 'reactxp';
import type { TransitionGroupProps } from '../transitions';

type TransitionContainerProps = {
  children: React.Node,
  ...TransitionGroupProps
};

type State = {
    previousChildren: ?React.Node,
    childrenAnimation: Types.AnimatedViewStyleRuleSet,
    previousChildrenAnimation: Types.AnimatedViewStyleRuleSet,
    animationStyles: Types.AnimatedViewStyleRuleSet,
    dimensions: Types.Dimensions,
};

const transitionContainerStyle = Styles.createViewStyle({
  flex: 1,
  position: 'relative'
});

export default class TransitionContainer extends Component<TransitionContainerProps, State> {

  constructor(props: TransitionContainerProps) {
    super(props);

    const dimensions = UserInterface.measureWindow();

    this.state = {
      dimensions,
      animationStyles: {
        style: Styles.createAnimatedViewStyle({
          position: 'absolute',
          width: dimensions.width,
          height: dimensions.height,
        }),
        allowPointerEvents: Styles.createAnimatedViewStyle({
          pointerEvents: 'auto',
        })
      }
    };
  }

  componentWillReceiveProps(nextProps: TransitionContainerProps) {
    switch (nextProps.name){
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
      childrenAnimation: this.state.animationStyles.allowPointerEvents,
      previousChildren: null
    });
  }

  slideUpTransition(nextProps: TransitionContainerProps) {
    const currentTranslationValue = Animated.createValue(this.state.dimensions.height);
    this.setState({
      previousChildren: this.props.children,
      childrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 1,
        transform: [{ translateY: currentTranslationValue }]
      }),
      previousChildrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 0,
        transform: [{ translateY: 0 }]
      }),
    }, () => {
      Animated.timing(currentTranslationValue, {
        toValue: 0,
        easing: Animated.Easing.InOut(),
        duration: nextProps.duration,
        useNativeDriver: false,
      }).start(() => this.onFinishedAnimation());
    });
  }

  slideDownTransition(nextProps: TransitionContainerProps) {
    const previousTranslationValue = Animated.createValue(0);
    this.setState({
      previousChildren: this.props.children,
      childrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 0,
        transform: [{ translateY: 0 }]
      }),
      previousChildrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 1,
        transform: [{ translateY: previousTranslationValue }]
      }),
    }, () => {
      Animated.timing(previousTranslationValue, {
        toValue: this.state.dimensions.height,
        easing: Animated.Easing.InOut(),
        duration: nextProps.duration,
        useNativeDriver: false,
      }).start(() => this.onFinishedAnimation());
    });
  }

  pushTransition(nextProps: TransitionContainerProps) {
    const currentTranslationValue = Animated.createValue(this.state.dimensions.width);
    const previousTranslationValue = Animated.createValue(0);
    this.setState({
      previousChildren: this.props.children,
      childrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 1,
        transform: [{ translateX: currentTranslationValue }]
      }),
      previousChildrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 0,
        transform: [{ translateX: previousTranslationValue }]
      }),
    }, () => {
      const compositeAnimation = Animated.parallel([
        Animated.timing(currentTranslationValue, {
          toValue: 0,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
          useNativeDriver: false,
        }),
        Animated.timing(previousTranslationValue, {
          toValue: - this.state.dimensions.width / 2,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
          useNativeDriver: false,
        })
      ]);
      compositeAnimation.start(() => this.onFinishedAnimation());
    });
  }

  popTransition(nextProps: TransitionContainerProps) {
    const currentTranslationValue = Animated.createValue(- this.state.dimensions.width / 2 );
    const previousTranslationValue = Animated.createValue(0);
    this.setState({
      previousChildren: this.props.children,
      childrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 0,
        transform: [{ translateX: currentTranslationValue }]
      }),
      previousChildrenAnimation: Styles.createAnimatedViewStyle({
        pointerEvents: 'none',
        zIndex: 1,
        transform: [{ translateX: previousTranslationValue }]
      }),
    }, () => {
      const compositeAnimation = Animated.parallel([
        Animated.timing(currentTranslationValue, {
          toValue: 0,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
          useNativeDriver: true,
        }),
        Animated.timing(previousTranslationValue, {
          toValue: this.state.dimensions.width,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
          useNativeDriver: true,
        })
      ]);
      compositeAnimation.start(() => this.onFinishedAnimation());
    });
  }

  render() {
    const { children } = this.props;
    const { previousChildren, childrenAnimation, previousChildrenAnimation } = this.state;
    return (
      <View style={[ transitionContainerStyle ]}>

        { previousChildren &&
          (<Animated.View key={ previousChildren && previousChildren.key }
            style={[this.state.animationStyles.style, previousChildrenAnimation]}>
            { previousChildren }
          </Animated.View>) }

        <Animated.View key={ children.key } style={[this.state.animationStyles.style, childrenAnimation]}>
          { children }
        </Animated.View>

      </View>
    );
  }
}