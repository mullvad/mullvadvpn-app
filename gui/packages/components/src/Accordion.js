// @flow

import * as React from 'react';
import { Component, View, Styles, Animated, UserInterface } from 'reactxp';

type Props = {
  height: number | 'auto',
  animationDuration?: number,
  children?: React.Node,
};

type State = {
  animatedValue: ?Animated.Value,
};

const containerOverflowStyle = Styles.createViewStyle({ overflow: 'hidden' });

export default class Accordion extends Component<Props, State> {
  static defaultProps = {
    height: 'auto',
    animationDuration: 350,
  };

  state: State = {
    animatedValue: null,
    animation: null,
  };

  _containerView: ?React.Node;
  _contentHeight = 0;
  _animation = (null: ?Animated.CompositeAnimation);

  constructor(props: Props) {
    super(props);

    // set the initial height if it's known
    if (typeof props.height === 'number') {
      this.state = {
        animatedValue: Animated.createValue(props.height),
      };
    }
  }

  componentWillUnmount() {
    if (this._animation) {
      this._animation.stop();
    }
  }

  shouldComponentUpdate(nextProps: Props, nextState: State) {
    return (
      nextState.animatedValue !== this.state.animatedValue ||
      nextProps.height !== this.props.height ||
      nextProps.children !== this.props.children
    );
  }

  componentDidUpdate(prevProps: Props, _prevState: State) {
    if (prevProps.height !== this.props.height) {
      this._animateHeightChanges();
    }
  }

  render() {
    const {
      style: style,
      height: _height,
      children,
      animationDuration: _animationDuration,
      ...otherProps
    } = this.props;
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
        ref={(node) => (this._containerView = node)}>
        <View onLayout={this._contentLayoutDidChange}>{children}</View>
      </Animated.View>
    );
  }

  async _animateHeightChanges() {
    const containerView = this._containerView;
    if (!containerView) {
      return;
    }

    if (this._animation) {
      this._animation.stop();
      this._animation = null;
    }

    try {
      const layout = await UserInterface.measureLayoutRelativeToWindow(containerView);
      const fromValue = this.state.animatedValue || Animated.createValue(layout.height);
      const toValue = this.props.height === 'auto' ? this._contentHeight : this.props.height;

      // calculate the animation duration based on travel distance
      const multiplier = Math.abs(toValue - layout.height) / Math.max(1, this._contentHeight);
      const duration = Math.ceil(this.props.animationDuration * multiplier);

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
    } catch (error) {
      // TODO: log error
    }
  }

  _onAnimationEnd = ({ finished }) => {
    if (finished) {
      this._animation = null;

      // reset height after transition to let element layout naturally
      // if animation finished without interruption
      if (this.props.height === 'auto') {
        this.setState({ animatedValue: null });
      }
    }
  };

  _contentLayoutDidChange = ({ height }) => (this._contentHeight = height);
}
