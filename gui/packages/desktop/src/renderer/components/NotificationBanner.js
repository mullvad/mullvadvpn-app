// @flow

import * as React from 'react';
import { Animated, View, Button, Text, Component, UserInterface, Styles, Types } from 'reactxp';
import { colors } from '../../config';
import Img from './Img';

const styles = {
  collapsible: Styles.createViewStyle({
    backgroundColor: 'rgba(25, 38, 56, 0.95)',
    overflow: 'hidden',
  }),
  drawer: Styles.createViewStyle({
    justifyContent: 'flex-end',
  }),
  container: Styles.createViewStyle({
    flexDirection: 'row',
    paddingTop: 8,
    paddingLeft: 20,
    paddingRight: 10,
    paddingBottom: 8,
  }),
  indicator: {
    base: Styles.createViewStyle({
      width: 10,
      height: 10,
      flex: 0,
      borderRadius: 5,
      marginTop: 4,
      marginRight: 8,
    }),
    warning: Styles.createViewStyle({
      backgroundColor: colors.yellow,
    }),
    success: Styles.createViewStyle({
      backgroundColor: colors.green,
    }),
    error: Styles.createViewStyle({
      backgroundColor: colors.red,
    }),
  },
  textContainer: Styles.createViewStyle({
    flex: 1,
  }),
  actionContainer: Styles.createViewStyle({
    flex: 0,
    flexDirection: 'column',
    justifyContent: 'center',
    marginLeft: 5,
  }),
  actionButton: Styles.createButtonStyle({
    flex: 1,
    justifyContent: 'center',
    cursor: 'default',
    paddingLeft: 5,
    paddingRight: 5,
  }),
  title: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    lineHeight: 18,
    color: colors.white,
  }),
  subtitle: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 18,
    color: colors.white60,
  }),
};

export class NotificationTitle extends Component {
  render() {
    return <Text style={styles.title}>{this.props.children}</Text>;
  }
}

export class NotificationSubtitle extends Component {
  render() {
    return React.Children.count(this.props.children) > 0 ? (
      <Text style={styles.subtitle}>{this.props.children}</Text>
    ) : null;
  }
}

export class NotificationOpenLinkAction extends Component<{ onPress: () => void }> {
  state = {
    hovered: false,
  };

  render() {
    return (
      <Button
        style={styles.actionButton}
        onPress={this.props.onPress}
        onHoverStart={this._onHoverStart}
        onHoverEnd={this._onHoverEnd}>
        <Img
          height={12}
          width={12}
          tintColor={this.state.hovered ? colors.white80 : colors.white60}
          source="icon-extLink"
        />
      </Button>
    );
  }

  _onHoverStart = () => {
    this.setState({ hovered: true });
  };

  _onHoverEnd = () => {
    this.setState({ hovered: false });
  };
}

export class NotificationContent extends Component {
  render() {
    return <View style={styles.textContainer}>{this.props.children}</View>;
  }
}

export class NotificationActions extends Component {
  render() {
    return <View style={styles.actionContainer}>{this.props.children}</View>;
  }
}

export class NotificationIndicator extends Component<{ type: 'success' | 'warning' | 'error' }> {
  render() {
    return <View style={[styles.indicator.base, styles.indicator[this.props.type]]} />;
  }
}

type NotificationBannerProps = {
  children: Array<
    React.Element<typeof NotificationContent> | React.Element<typeof NotificationActions>,
  >,
  visible: boolean,
  animationDuration: number,
};

type NotificationBannerState = {
  contentPinnedToBottom: boolean,
};

export class NotificationBanner extends Component<
  NotificationBannerProps,
  NotificationBannerState,
> {
  static defaultProps = {
    animationDuration: 350,
  };

  _containerRef = React.createRef();
  _contentHeight = 0;
  _heightValue = Animated.createValue(0);
  _animationStyle: Types.AnimatedViewStyle;
  _animation: ?Types.Animated.CompositeAnimation = null;
  _didFinishFirstLayoutPass = false;

  state = {
    contentPinnedToBottom: false,
  };

  constructor(props: NotificationBannerProps) {
    super(props);

    this._animationStyle = Styles.createAnimatedViewStyle({
      height: this._heightValue,
    });
  }

  shouldComponentUpdate(nextProps: NotificationBannerProps, nextState: NotificationBannerState) {
    return (
      this.props.children !== nextProps.children ||
      this.props.visible !== nextProps.visible ||
      this.state.contentPinnedToBottom !== nextState.contentPinnedToBottom
    );
  }

  componentDidUpdate(prevProps: NotificationBannerProps) {
    if (prevProps.visible !== this.props.visible) {
      // enable drawer-like animation when changing banner's visibility
      this.setState({ contentPinnedToBottom: true }, () => {
        this._animateHeightChanges();
      });
    }
  }

  componentWillUnmount() {
    if (this._animation) {
      this._animation.stop();
    }
  }

  render() {
    return (
      <Animated.View
        style={[
          styles.collapsible,
          this.state.contentPinnedToBottom ? styles.drawer : undefined,
          this._animationStyle,
          this.props.style,
        ]}
        ref={this._containerRef}>
        <View onLayout={this._onLayout}>
          <View style={styles.container}>{this.props.children}</View>
        </View>
      </Animated.View>
    );
  }

  _onLayout = ({ height }) => {
    const oldHeight = this._contentHeight;
    this._contentHeight = height;

    // The first layout pass should not be animated because this would cause the initially visible
    // notification banner to slide down each time the component is mounted.
    if (this._didFinishFirstLayoutPass) {
      if (oldHeight !== height) {
        this._animateHeightChanges();
      }
    } else {
      this._didFinishFirstLayoutPass = true;
      if (this.props.visible) {
        this._heightValue.setValue(height);
      }
    }
  };

  async _animateHeightChanges() {
    const containerView = this._containerRef.current;
    if (!containerView) {
      return;
    }

    if (this._animation) {
      this._animation.stop();
      this._animation = null;
    }

    // calculate the animation duration based on travel distance
    const layout = await UserInterface.measureLayoutRelativeToWindow(containerView);
    const toValue = this.props.visible ? this._contentHeight : 0;
    const multiplier = Math.abs(toValue - layout.height) / Math.max(1, this._contentHeight);
    const duration = Math.ceil(this.props.animationDuration * multiplier);

    const animation = Animated.timing(this._heightValue, {
      toValue,
      easing: Animated.Easing.InOut(),
      duration,
      useNativeDriver: true,
    });

    this._animation = animation;

    animation.start(({ finished }) => {
      if (finished) {
        // disable drawer-like animations for content updates when the banner is visible
        this.setState({ contentPinnedToBottom: false });
      }
    });
  }
}
