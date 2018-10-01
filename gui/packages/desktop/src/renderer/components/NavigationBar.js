// @flow

import * as React from 'react';
import { Animated, Button, Component, Text, View, Styles, UserInterface } from 'reactxp';
import Img from './Img';
import CustomScrollbars from './CustomScrollbars';
import { colors } from '../../config';

const styles = {
  navigationBar: {
    default: Styles.createViewStyle({
      flex: 0,
      flexDirection: 'row',
      paddingHorizontal: 12,
      paddingBottom: 12,
    }),
    separator: Styles.createViewStyle({
      borderStyle: 'solid',
      borderBottomWidth: 1,
      borderColor: 'rgba(0, 0, 0, 0.2)',
    }),
    darwin: Styles.createViewStyle({
      paddingTop: 24,
    }),
    win32: Styles.createViewStyle({
      paddingTop: 12,
    }),
    linux: Styles.createViewStyle({
      paddingTop: 12,
      WebkitAppRegion: 'drag',
    }),
  },
  navigationBarTitle: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'column',
    justifyContent: 'center',
  }),
  titleBarItem: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '600',
    lineHeight: 22,
    color: colors.white60,
    alignSelf: 'center',
  }),
  closeBarItem: {
    default: Styles.createViewStyle({
      cursor: 'default',
      WebkitAppRegion: 'no-drag',
    }),
    icon: Styles.createViewStyle({
      flex: 0,
      opacity: 0.6,
    }),
  },
  backBarButton: {
    default: Styles.createViewStyle({
      borderWidth: 0,
      padding: 0,
      margin: 0,
      cursor: 'default',
      WebkitAppRegion: 'no-drag',
    }),
    content: Styles.createViewStyle({
      flexDirection: 'row',
      alignItems: 'center',
    }),
    label: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      color: colors.white60,
    }),
    icon: Styles.createViewStyle({
      opacity: 0.6,
      marginRight: 8,
    }),
  },
};

const NavigationScrollContext = React.createContext({
  scrollTop: 0,
  onScroll: (_scroll) => {},
});

export class NavigationContainer extends Component {
  state = {
    scrollTop: 0,
  };

  _onScroll = (scroll) => {
    this.setState({
      scrollTop: scroll.scrollTop,
    });
  };

  render() {
    return (
      <NavigationScrollContext.Provider
        value={{ scrollTop: this.state.scrollTop, onScroll: this._onScroll }}>
        {this.props.children}
      </NavigationScrollContext.Provider>
    );
  }
}

export const NavigationScrollbars = React.forwardRef(function NavigationScrollbars(props, ref) {
  return (
    <NavigationScrollContext.Consumer>
      {(context) => {
        const { children, ...otherProps } = props;
        const wrappedOnScroll = (scroll) => {
          context.onScroll(scroll);

          if (otherProps.onScroll) {
            otherProps.onScroll(scroll);
          }
        };

        return (
          <CustomScrollbars {...otherProps} ref={ref} onScroll={wrappedOnScroll}>
            {children}
          </CustomScrollbars>
        );
      }}
    </NavigationScrollContext.Consumer>
  );
});

type NavigationBarTitleProps = {
  visible: boolean,
  titleAdjustment: number,
  children?: React.Node,
};

class NavigationBarTitle extends Component<NavigationBarTitleProps> {
  render() {
    const titleAdjustment = this.props.titleAdjustment;
    const titleAdjustmentStyle = Styles.createViewStyle(
      {
        paddingRight: titleAdjustment > 0 ? titleAdjustment : 0,
        paddingLeft: titleAdjustment < 0 ? Math.abs(titleAdjustment) : 0,
      },
      false,
    );

    return (
      <View style={[styles.navigationBarTitle, titleAdjustmentStyle]}>
        <NavigationBarAnimatedTitle visible={this.props.visible}>
          {this.props.children}
        </NavigationBarAnimatedTitle>
      </View>
    );
  }
}

type NavigationBarAnimatedTitleProps = {
  visible: boolean,
  children?: React.Node,
};

class NavigationBarAnimatedTitle extends Component<NavigationBarAnimatedTitleProps> {
  _opacityValue: Animated.Value;
  _opacityStyle: Styles.AnimatedViewStyle;
  _animation: ?Animated.Animation;

  constructor(props: NavigationBarAnimatedTitleProps) {
    super(props);

    this._opacityValue = Animated.createValue(props.visible ? 1 : 0);
    this._opacityStyle = Styles.createAnimatedViewStyle({
      opacity: this._opacityValue,
    });
  }

  shouldComponentUpdate(nextProps) {
    return this.props.visible !== nextProps.visible || this.props.children !== nextProps.children;
  }

  componentDidUpdate() {
    this._animateOpacity(this.props.visible);
  }

  componentWillUnmount() {
    if (this._animation) {
      this._animation.stop();
    }
  }

  render() {
    return <Animated.View style={this._opacityStyle}>{this.props.children}</Animated.View>;
  }

  _animateOpacity(visible: boolean) {
    const oldAnimation = this._animation;
    if (oldAnimation) {
      oldAnimation.stop();
    }

    const animation = Animated.timing(this._opacityValue, {
      toValue: visible ? 1 : 0,
      easing: Animated.Easing.InOut(),
      duration: 250,
    });

    animation.start();

    this._animation = animation;
  }
}

type NavigationBarState = {
  titleAdjustment: number,
};

export class NavigationBar extends Component<{}, NavigationBarState> {
  state = {
    titleAdjustment: 0,
  };

  _titleViewRef = React.createRef();

  _onLayout = async (containerLayout) => {
    const titleView = this._titleViewRef.current;
    if (titleView) {
      // calculate the title layout frame
      const titleLayout = await UserInterface.measureLayoutRelativeToAncestor(
        this._titleViewRef.current,
        this,
      );

      // calculate the remaining space at the right hand side
      const trailingSpace = containerLayout.width - (titleLayout.x + titleLayout.width);

      this.setState({
        titleAdjustment: titleLayout.x - trailingSpace,
      });
    }
  };

  render() {
    return (
      <NavigationScrollContext.Consumer>
        {(context) => (
          <View
            style={[
              styles.navigationBar.default,
              this._shouldShowBarSeparator(context.scrollTop) && styles.navigationBar.separator,
              styles.navigationBar[process.platform],
            ]}
            onLayout={this._onLayout}>
            {React.Children.map(this.props.children, (element) => {
              if (element.type === TitleBarItem) {
                return (
                  <NavigationBarTitle
                    titleAdjustment={this.state.titleAdjustment}
                    visible={this._shouldShowNavigationTitle(context.scrollTop)}
                    ref={this._titleViewRef}>
                    {element}
                  </NavigationBarTitle>
                );
              } else {
                return <View>{element}</View>;
              }
            })}
          </View>
        )}
      </NavigationScrollContext.Consumer>
    );
  }

  _shouldShowBarSeparator(scrollTop: number): boolean {
    // that's where SettingsHeader.HeaderTitle intersects the navigation bar
    return scrollTop > 11;
  }

  _shouldShowNavigationTitle(scrollTop: number): boolean {
    // that's when SettingsHeader.HeaderTitle goes behind the navigation bar
    return scrollTop > 30;
  }
}

export class TitleBarItem extends Component {
  render() {
    return <Text style={styles.titleBarItem}>{this.props.children}</Text>;
  }
}

export class CloseBarItem extends Component {
  props: {
    action: () => void,
  };
  render() {
    return (
      <Button style={[styles.closeBarItem.default]} onPress={this.props.action}>
        <Img height={24} width={24} style={[styles.closeBarItem.icon]} source="icon-close" />
      </Button>
    );
  }
}

export class BackBarItem extends Component {
  props: {
    title: string,
    action: () => void,
  };
  render() {
    return (
      <Button style={styles.backBarButton.default} onPress={this.props.action}>
        <View style={styles.backBarButton.content}>
          <Img style={styles.backBarButton.icon} source="icon-back" />
          <Text style={styles.backBarButton.label}>{this.props.title}</Text>
        </View>
      </Button>
    );
  }
}
