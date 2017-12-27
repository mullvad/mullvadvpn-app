// @flow

import * as React from 'react';
import { Component, View, Styles, Animated } from 'reactxp';

export type AccordionProps = {
  height: number | 'auto',
  animationDuration?: number,
  children?: React.Node
};

export type AccordionState = {
  animatedValue: ?Animated.Value,
};

const containerOverflowStyle = Styles.createViewStyle({ overflow: 'hidden' });

export default class Accordion extends Component<AccordionProps, AccordionState> {
  static defaultProps = {
    height: 'auto',
    animationDuration: 250
  };

  state: AccordionState = {
    animatedValue: null,
    animation: null,
  };

  _containerHeight = 0;
  _contentHeight = 0;
  _animation = (null: ?Animated.CompositeAnimation);

  constructor(props: AccordionProps) {
    super(props);

    // set the initial height if it's known
    const initialHeight = props.height;
    if(typeof(initialHeight) === 'number') {
      this._containerHeight = initialHeight;
      this.state = {
        animatedValue: Animated.createValue(initialHeight)
      };
    }
  }

  componentWillUnmount() {
    if(this._animation) {
      this._animation.stop();
    }
  }

  componentDidUpdate(prevProps: AccordionProps, _prevState: AccordionState) {
    if(prevProps.height !== this.props.height) {
      this._animateHeightChanges();
    }
  }

  render() {
    const { height: _height, children, animationDuration: _animationDuration, ...otherProps } = this.props;
    const containerStyles = [];

    if(this.state.animatedValue !== null) {
      const animatedStyle = Styles.createAnimatedViewStyle({
        height: this.state.animatedValue,
      });

      containerStyles.push(containerOverflowStyle, animatedStyle);
    }

    return (
      <Animated.View { ...otherProps } style={ containerStyles } onLayout={ this._containerLayoutDidChange }>
        <View onLayout={ this._contentLayoutDidChange }>
          { children }
        </View>
      </Animated.View>
    );
  }

  _animateHeightChanges() {
    // call stop to get updated fromValue._value
    if(this._animation) {
      this._animation.stop();
    }

    const fromValue = this.state.animatedValue || Animated.createValue(this._containerHeight);
    const toValue = this.props.height === 'auto' ? this._contentHeight : this.props.height;

    // calculate the animation duration based on travel distance
    // note: _getValue() is private.
    const primitiveFromValue = parseInt(fromValue._getValue());
    const multiplier = Math.abs(toValue - primitiveFromValue) / Math.max(1, this._contentHeight);
    const duration = this.props.animationDuration * multiplier;
    const animation = Animated.timing(fromValue, {
      toValue: toValue,
      easing: Animated.Easing.InOut(),
      duration: duration,
      useNativeDriver: true,
    });

    this._animation = animation;
    this.setState({ animatedValue: fromValue }, () => {
      animation.start(this._onAnimationEnd);
    });
  }

  _onAnimationEnd = ({ finished }) => {
    // reset height after transition to let element layout naturally
    // if animation finished without interruption
    if(this.props.height === 'auto' && finished) {
      this.setState({ animatedValue: null });
    }
  }

  _containerLayoutDidChange = ({ height }) => this._containerHeight = height;
  _contentLayoutDidChange = ({ height }) => this._contentHeight = height;
}