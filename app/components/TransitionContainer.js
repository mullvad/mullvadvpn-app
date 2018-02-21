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
    currentTranslationValue: Animated.Value,
    previousTranslationValue: Animated.Value,
    toValue: number,
    dimensions: Types.Dimensions,
};

export default class TransitionContainer extends Component<TransitionContainerProps, State>{

  constructor(props: TransitionContainerProps) {
    super(props);
    const dimensions = UserInterface.measureWindow();
    this.state = {
      dimensions: dimensions,
      currentTranslationValue: Animated.createValue(0),
      previousTranslationValue: Animated.createValue(0),
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
      this.state.currentTranslationValue.setValue(this.state.dimensions.height);
      this.setState({
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateY: this.state.currentTranslationValue }]
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{ translateY: 0 }]
        }),
      }, ()=>{
        Animated.timing(this.state.currentTranslationValue, {
          toValue: 0,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
          useNativeDriver: true,
        }).start(()=>{
          this.setState({
            childrenAnimation: this.state.animationStyles.allowPointerEvents,
            previousChildren: null
          });
        });
      });
      break;
    case 'slide-down':
      this.state.previousTranslationValue.setValue(0);
      this.setState({
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{
            translateY: 0 }]
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateY: this.state.previousTranslationValue }]
        }),
      }, ()=>{
        Animated.timing(this.state.previousTranslationValue, {
          toValue: this.state.dimensions.height,
          easing: Animated.Easing.InOut(),
          duration: nextProps.duration,
          useNativeDriver: true,
        }).start(()=>{
          this.setState({
            childrenAnimation: this.state.animationStyles.allowPointerEvents,
            previousChildren: null
          });
        });
      });
      break;
    case 'push':
      this.state.currentTranslationValue.setValue(this.state.dimensions.width);
      this.state.previousTranslationValue.setValue(0);
      this.setState({
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateX: this.state.currentTranslationValue }]
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{ translateX: this.state.previousTranslationValue }]
        }),
      }, ()=>{
        const compositeAnimation = Animated.parallel([
          Animated.timing(this.state.currentTranslationValue, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
            useNativeDriver: true,
          }),
          Animated.timing(this.state.previousTranslationValue, {
            toValue: - this.state.dimensions.width / 2,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
            useNativeDriver: true,
          })
        ]);
        compositeAnimation.start(() => this.setState({
          childrenAnimation: this.state.animationStyles.allowPointerEvents,
          previousChildren: null
        }));
      });
      break;
    case 'pop':
      this.state.currentTranslationValue.setValue(- this.state.dimensions.width / 2 );
      this.state.previousTranslationValue.setValue(0);
      this.setState({
        previousChildren: this.props.children,
        childrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 0,
          transform: [{ translateX: this.state.currentTranslationValue }]
        }),
        previousChildrenAnimation: Styles.createAnimatedViewStyle({
          pointerEvents: 'none',
          zIndex: 1,
          transform: [{ translateX: this.state.previousTranslationValue }]
        }),
      }, ()=>{
        const compositeAnimation = Animated.parallel([
          Animated.timing(this.state.currentTranslationValue, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
            useNativeDriver: true,
          }),
          Animated.timing(this.state.previousTranslationValue, {
            toValue: this.state.dimensions.width,
            easing: Animated.Easing.InOut(),
            duration: nextProps.duration,
            useNativeDriver: true,
          })
        ]);
        compositeAnimation.start(() => this.setState({
          childrenAnimation: this.state.animationStyles.allowPointerEvents,
          previousChildren: null
        }));
      });
      break;
    default:
      break;
    }

  }

  render() {
    const { children } = this.props;
    const { previousChildren, childrenAnimation, previousChildrenAnimation} = this.state;
    return (
      <View style={{flex:1}}>
        <Animated.View key={previousChildren ? previousChildren.key : null } style={[this.state.animationStyles.style, previousChildrenAnimation]}>
          {previousChildren}
        </Animated.View>
        <Animated.View key={children.key} style={[this.state.animationStyles.style, childrenAnimation]}>
          {children}
        </Animated.View>
      </View>
    );
  }
}