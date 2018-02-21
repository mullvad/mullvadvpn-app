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
    currentTranslationValue: Animated.createValue(0),
    previousTranslationValue: Animated.createValue(0),
    toValue: 0,
    dimensions: UserInterface.measureWindow(),
  }

  animationStyles = {
    style: Styles.createAnimatedViewStyle({
      position: 'absolute',
      width: this.state.dimensions.width,
      height: this.state.dimensions.height,
    }),
    allowPointerEvents: Styles.createAnimatedViewStyle({
      pointerEvents: 'auto',
    }),
  }

  componentWillReceiveProps(nextProps: TransitionContainerProps) {
    if (this.props.children.key !== nextProps.children.key){
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
              childrenAnimation: this.animationStyles.allowPointerEvents,
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
              childrenAnimation: this.animationStyles.allowPointerEvents,
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
          let compositeAnimation = Animated.parallel([
            Animated.timing(this.state.currentTranslationValue, {
              toValue: 0,
              easing: Animated.Easing.InOut(),
              duration: nextProps.duration,
              useNativeDriver: true,
            }),
            Animated.timing(this.state.previousTranslationValue, {
              toValue: -50,
              easing: Animated.Easing.InOut(),
              duration: nextProps.duration,
              useNativeDriver: true,
            })
          ]);
          compositeAnimation.start(() => this.setState({
            childrenAnimation: this.animationStyles.allowPointerEvents,
            previousChildren: null
          }));
        });
        break;
      case 'pop':
        this.state.currentTranslationValue.setValue(-50);
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
          let compositeAnimation = Animated.parallel([
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
            childrenAnimation: this.animationStyles.allowPointerEvents,
            previousChildren: null
          }));
        });
        break;
      default:
        break;
      }
    }
  }

  render() {
    const { children } = this.props;
    const { previousChildren, childrenAnimation, previousChildrenAnimation} = this.state;

    return (
      <View style={{flex:1}}>
        <Animated.View key={previousChildren ? previousChildren.key : null } style={[this.animationStyles.style, previousChildrenAnimation]}>
          {previousChildren}
        </Animated.View>
        <Animated.View key={children.key} style={[this.animationStyles.style, childrenAnimation]}>
          {children}
        </Animated.View>
      </View>
    );
  }
}