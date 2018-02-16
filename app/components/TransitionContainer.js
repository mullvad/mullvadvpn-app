// @flow
import * as React from 'react';
import { Styles, Component, Animated, View, Types, UserInterface } from 'reactxp';
import type { TransitionGroupProps } from '../transitions';

export type TransitionContainerProps = {
    children: React.Node,
    ...TransitionGroupProps
  }

export default class TransitionContainer extends Component {
  props: TransitionContainerProps;

  state = {
    previousChildren: React.Node,
    childrenAnimation: Types.AnimatedViewStyleRuleSet,
    previousChildrenAnimation: Types.AnimatedViewStyleRuleSet,
    translationValue: Animated.createValue(0),
    toValue: 0,
    dimensions: UserInterface.measureWindow(),
    style: Types.AnimatedViewStyleRuleSet,
    slideXAnimationStyle: Types.AnimatedViewStyleRuleSet,
    noXAnimationStyle: Types.AnimatedViewStyleRuleSet,
    slideYAnimationStyle: Types.AnimatedViewStyleRuleSet,
    noYAnimationStyle: Types.AnimatedViewStyleRuleSet,
  }

  animationStyles = {
    style: Styles.createAnimatedViewStyle({
      position: 'absolute',
      width: this.state.dimensions.width,
      height: this.state.dimensions.height,
    }),
    slideYAnimationStyle: Styles.createAnimatedViewStyle({
      pointerEvents: 'none',
      zIndex: 1,
      transform: [{
        translateY: this.state.translationValue }]
    }),
    slideXAnimationStyle: Styles.createAnimatedViewStyle({
      pointerEvents: 'none',
      zIndex: 1,
      transform: [{
        translateX: this.state.translationValue }]
    }),
    noYAnimationStyle: Styles.createAnimatedViewStyle({
      pointerEvents: 'none',
      zIndex: 0,
      transform: [{
        translateY: 0 }]
    }),
    noXAnimationStyle: Styles.createAnimatedViewStyle({
      pointerEvents: 'none',
      zIndex: 0,
      transform: [{
        translateX: 0 }]
    }),
    allowPointerEvents: Styles.createAnimatedViewStyle({
      pointerEvents: 'auto',
    }),
  }

  componentWillReceiveProps(nextProps: TransitionContainerProps) {
    if ((this.props.children.key !== nextProps.children.key) && nextProps.transitionEnter){
      switch (nextProps.transitionName){
      case 'slide-up-transition':
        this.state.translationValue.setValue(this.state.dimensions.height);
        this.setState({
          previousChildren: this.props.children,
          childrenAnimation: this.animationStyles.slideYAnimationStyle,
          previousChildrenAnimation: this.animationStyles.noYAnimationStyle,
        }, ()=>{
          Animated.timing(this.state.translationValue, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration: nextProps.transitionDuration,
            useNativeDriver: true,
          }
          ).start(()=>{
            this.setState({
              childrenAnimation: this.animationStyles.allowPointerEvents,
              previousChildren: null
            });
          });
        });
        break;
      case 'slide-down-transition':
        this.state.translationValue.setValue(0);
        this.setState({
          previousChildren: this.props.children,
          childrenAnimation: this.animationStyles.noYAnimationStyle,
          previousChildrenAnimation: this.animationStyles.slideYAnimationStyle,
        }, ()=>{
          Animated.timing(this.state.translationValue, {
            toValue: this.state.dimensions.height,
            easing: Animated.Easing.InOut(),
            duration: nextProps.transitionDuration,
            useNativeDriver: true,
          }
          ).start(()=>{
            this.setState({
              childrenAnimation: this.animationStyles.allowPointerEvents,
              previousChildren: null
            });
          });
        });
        break;
      case 'push-transition':
        this.state.translationValue.setValue(this.state.dimensions.width);
        this.setState({
          previousChildren: this.props.children,
          childrenAnimation: this.animationStyles.slideXAnimationStyle,
          previousChildrenAnimation: this.animationStyles.noXAnimationStyle,
        }, ()=>{
          Animated.timing(this.state.translationValue, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration: nextProps.transitionDuration,
            useNativeDriver: true,
          }
          ).start(()=>{
            this.setState({
              childrenAnimation: this.animationStyles.allowPointerEvents,
              previousChildren: null
            });
          });
        });
        break;
      case 'pop-transition':
        this.state.translationValue.setValue(0);
        this.setState({
          previousChildren: this.props.children,
          childrenAnimation: this.animationStyles.noXAnimationStyle,
          previousChildrenAnimation: this.animationStyles.slideXAnimationStyle,
        }, ()=>{
          Animated.timing(this.state.translationValue, {
            toValue: this.state.dimensions.width,
            easing: Animated.Easing.InOut(),
            duration: nextProps.transitionDuration,
            useNativeDriver: true,
          }
          ).start(()=>{
            this.setState({
              childrenAnimation: this.animationStyles.allowPointerEvents,
              previousChildren: null
            });
          });
        });
        break;
      }
    }
  }

  render() {
    const { children } = this.props;
    const { previousChildren, childrenAnimation, previousChildrenAnimation} = this.state;

    return (
      <View style={{flex:1}}>
        <Animated.View style={[this.animationStyles.style, previousChildrenAnimation]}>
          {previousChildren}
        </Animated.View>
        <Animated.View style={[this.animationStyles.style, childrenAnimation]}>
          {children}
        </Animated.View>
      </View>
    );
  }
}