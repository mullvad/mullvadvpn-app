import * as React from 'react';
import { Animated, Button, Component, Styles, Text, Types, UserInterface, View } from 'reactxp';
import { colors } from '../../config.json';
import CustomScrollbars, { IScrollEvent } from './CustomScrollbars';
import ImageView from './ImageView';

const styles = {
  navigationBar: {
    default: Styles.createViewStyle({
      flex: 0,
      paddingHorizontal: 12,
      paddingBottom: 12,
    }),
    content: Styles.createViewStyle({
      flex: 1,
      flexDirection: 'row',
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
    }),
  },
  navigationBarTitle: {
    container: Styles.createViewStyle({
      flex: 1,
      flexDirection: 'column',
      justifyContent: 'center',
    }),
    label: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 16,
      fontWeight: '600',
      lineHeight: 22,
      color: colors.white,
      paddingHorizontal: 5,
      textAlign: 'center',
    }),
    measuringLabel: Styles.createTextStyle({
      position: 'absolute',
      opacity: 0,
    }),
  },
  buttonBarItem: {
    default: Styles.createButtonStyle({
      cursor: 'default',
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
  },
  closeBarItem: {
    default: Styles.createViewStyle({
      cursor: 'default',
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

interface INavigationScrollContextValue {
  scrollTop: number;
  onScroll: (event: IScrollEvent) => void;
}

const NavigationScrollContext = React.createContext<INavigationScrollContextValue>({
  scrollTop: 0,
  onScroll: (_event: IScrollEvent) => {
    // no-op
  },
});

export class NavigationContainer extends Component {
  public state = {
    scrollTop: 0,
  };

  public render() {
    return (
      <NavigationScrollContext.Provider
        value={{ scrollTop: this.state.scrollTop, onScroll: this.onScroll }}>
        {this.props.children}
      </NavigationScrollContext.Provider>
    );
  }

  private onScroll = (event: IScrollEvent) => {
    this.setState({
      scrollTop: event.scrollTop,
    });
  };
}

interface INavigationScrollbarsProps {
  onScroll?: (value: IScrollEvent) => void;
  style?: React.CSSProperties;
  children?: React.ReactNode;
}
export const NavigationScrollbars = React.forwardRef(function NavigationScrollbarsT(
  props: INavigationScrollbarsProps,
  ref?: React.Ref<CustomScrollbars>,
) {
  return (
    <NavigationScrollContext.Consumer>
      {(context) => {
        const { style, children, ...otherProps } = props;
        const wrappedOnScroll = (scroll: IScrollEvent) => {
          context.onScroll(scroll);

          if (otherProps.onScroll) {
            otherProps.onScroll(scroll);
          }
        };

        return (
          <CustomScrollbars ref={ref} style={style} onScroll={wrappedOnScroll}>
            {children}
          </CustomScrollbars>
        );
      }}
    </NavigationScrollContext.Consumer>
  );
});

interface IPrivateTitleBarItemProps {
  visible: boolean;
  titleAdjustment: number;
  measuringTextRef?: React.RefObject<Text>;
  children?: React.ReactText;
}

class PrivateTitleBarItem extends Component<IPrivateTitleBarItemProps> {
  public shouldComponentUpdate(nextProps: IPrivateTitleBarItemProps) {
    return (
      this.props.visible !== nextProps.visible ||
      this.props.titleAdjustment !== nextProps.titleAdjustment ||
      this.props.children !== nextProps.children
    );
  }

  public render() {
    const titleAdjustment = this.props.titleAdjustment;
    const titleAdjustmentStyle = Styles.createViewStyle({ marginLeft: titleAdjustment }, false);

    return (
      <View style={styles.navigationBarTitle.container}>
        <PrivateBarItemAnimationContainer visible={this.props.visible}>
          <Text
            style={[styles.navigationBarTitle.label, titleAdjustmentStyle]}
            ellipsizeMode="tail"
            numberOfLines={1}>
            {this.props.children}
          </Text>
        </PrivateBarItemAnimationContainer>

        <Text
          style={[styles.navigationBarTitle.label, styles.navigationBarTitle.measuringLabel]}
          numberOfLines={1}
          ref={this.props.measuringTextRef}>
          {this.props.children}
        </Text>
      </View>
    );
  }
}

interface IPrivateBarItemAnimationContainerProps {
  visible: boolean;
  children?: React.ReactNode;
}

class PrivateBarItemAnimationContainer extends Component<IPrivateBarItemAnimationContainerProps> {
  private opacityValue: Animated.Value;
  private opacityStyle: Types.AnimatedViewStyleRuleSet;
  private animation?: Types.Animated.CompositeAnimation;

  constructor(props: IPrivateBarItemAnimationContainerProps) {
    super(props);

    this.opacityValue = Animated.createValue(props.visible ? 1 : 0);
    this.opacityStyle = Styles.createAnimatedViewStyle({
      opacity: this.opacityValue,
    });
  }

  public shouldComponentUpdate(nextProps: IPrivateBarItemAnimationContainerProps) {
    return this.props.visible !== nextProps.visible || this.props.children !== nextProps.children;
  }

  public componentDidUpdate() {
    this.animateOpacity(this.props.visible);
  }

  public componentWillUnmount() {
    if (this.animation) {
      this.animation.stop();
    }
  }

  public render() {
    return <Animated.View style={this.opacityStyle}>{this.props.children}</Animated.View>;
  }

  private animateOpacity(visible: boolean) {
    const oldAnimation = this.animation;
    if (oldAnimation) {
      oldAnimation.stop();
    }

    const animation = Animated.timing(this.opacityValue, {
      toValue: visible ? 1 : 0,
      easing: Animated.Easing.InOut(),
      duration: 250,
    });

    animation.start();

    this.animation = animation;
  }
}

interface INavigationBarProps {
  children?: React.ReactNode;
}

export const NavigationBar = React.forwardRef(function NavigationBarT(
  props: INavigationBarProps,
  ref?: React.Ref<PrivateNavigationBar>,
) {
  return (
    <NavigationScrollContext.Consumer>
      {(context) => (
        <PrivateNavigationBar ref={ref} scrollTop={context.scrollTop}>
          {props.children}
        </PrivateNavigationBar>
      )}
    </NavigationScrollContext.Consumer>
  );
});

interface IPrivateNavigationBarProps {
  scrollTop: number;
  children?: React.ReactNode;
}

interface IPrivateNavigationBarState {
  titleAdjustment: number;
  showsBarSeparator: boolean;
  showsBarTitle: boolean;
}

const PrivateTitleBarItemContext = React.createContext({
  titleAdjustment: 0,
  visible: false,
  titleRef: React.createRef<PrivateTitleBarItem>(),
  measuringTextRef: React.createRef<Text>(),
});

class PrivateNavigationBar extends Component<
  IPrivateNavigationBarProps,
  IPrivateNavigationBarState
> {
  public static defaultProps: Partial<IPrivateNavigationBarProps> = {
    scrollTop: 0,
  };

  public static getDerivedStateFromProps(
    props: IPrivateNavigationBarProps,
    state: IPrivateNavigationBarState,
  ) {
    // that's where SettingsHeader.HeaderTitle intersects the navigation bar
    const showsBarSeparator = props.scrollTop > 11;

    // that's when SettingsHeader.HeaderTitle goes behind the navigation bar
    const showsBarTitle = props.scrollTop > 30;

    return {
      ...state,
      showsBarSeparator,
      showsBarTitle,
    };
  }

  public state: IPrivateNavigationBarState = {
    titleAdjustment: 0,
    showsBarSeparator: false,
    showsBarTitle: false,
  };

  private titleViewRef = React.createRef<PrivateTitleBarItem>();
  private measuringTextRef = React.createRef<Text>();

  public shouldComponentUpdate(
    nextProps: IPrivateNavigationBarProps,
    nextState: IPrivateNavigationBarState,
  ) {
    return (
      this.props.children !== nextProps.children ||
      this.state.titleAdjustment !== nextState.titleAdjustment ||
      this.state.showsBarSeparator !== nextState.showsBarSeparator ||
      this.state.showsBarTitle !== nextState.showsBarTitle
    );
  }

  public render() {
    return (
      <View
        style={[
          styles.navigationBar.default,
          this.state.showsBarSeparator ? styles.navigationBar.separator : undefined,
          this.getPlatformStyle(),
        ]}>
        <View style={styles.navigationBar.content} onLayout={this.onLayout}>
          <PrivateTitleBarItemContext.Provider
            value={{
              titleAdjustment: this.state.titleAdjustment,
              visible: this.state.showsBarTitle,
              titleRef: this.titleViewRef,
              measuringTextRef: this.measuringTextRef,
            }}>
            {this.props.children}
          </PrivateTitleBarItemContext.Provider>
        </View>
      </View>
    );
  }

  private getPlatformStyle(): Types.ViewStyleRuleSet | undefined {
    switch (process.platform) {
      case 'darwin':
        return styles.navigationBar.darwin;
      case 'win32':
        return styles.navigationBar.win32;
      case 'linux':
        return styles.navigationBar.linux;
      default:
        return undefined;
    }
  }

  private onLayout = async (navBarContentLayout: Types.ViewOnLayoutEvent) => {
    const titleViewContainer = this.titleViewRef.current;
    const measuringText = this.measuringTextRef.current;

    if (titleViewContainer && measuringText) {
      const titleLayout = await UserInterface.measureLayoutRelativeToAncestor(
        titleViewContainer,
        this,
      );
      const textLayout = await UserInterface.measureLayoutRelativeToAncestor(measuringText, this);

      // calculate the width of the elements preceding the title view container
      const leadingSpace = titleLayout.x - navBarContentLayout.x;

      // calculate the width of the elements succeeding the title view container
      const trailingSpace = navBarContentLayout.width - titleLayout.width - leadingSpace;

      // calculate the adjustment needed to center the title view within navigation bar
      const titleAdjustment = Math.floor(trailingSpace - leadingSpace);

      // calculate the maximum possible adjustment that when applied should keep the text fully
      // visible, unless the title container itself is smaller than the space needed to accommodate
      // the text
      const maxTitleAdjustment = Math.floor(Math.max(titleLayout.width - textLayout.width, 0));

      // cap the adjustment to remain within the allowed bounds
      const cappedTitleAdjustment = Math.min(
        Math.max(-maxTitleAdjustment, titleAdjustment),
        maxTitleAdjustment,
      );

      if (this.state.titleAdjustment !== cappedTitleAdjustment) {
        this.setState({
          titleAdjustment: cappedTitleAdjustment,
        });
      }
    }
  };
}

interface ITitleBarItemProps {
  children?: React.ReactText;
}
export function TitleBarItem(props: ITitleBarItemProps) {
  return (
    <PrivateTitleBarItemContext.Consumer>
      {(context) => (
        <PrivateTitleBarItem
          titleAdjustment={context.titleAdjustment}
          visible={context.visible}
          ref={context.titleRef}
          measuringTextRef={context.measuringTextRef}>
          {props.children}
        </PrivateTitleBarItem>
      )}
    </PrivateTitleBarItemContext.Consumer>
  );
}

interface ICloseBarItemProps {
  action: () => void;
}

export class CloseBarItem extends Component<ICloseBarItemProps> {
  public render() {
    // Use the arrow down icon on Linux, to avoid confusion with the close button in the window
    // title bar.
    const iconName = process.platform === 'linux' ? 'icon-close-down' : 'icon-close';

    return (
      <Button style={[styles.closeBarItem.default]} onPress={this.props.action}>
        <ImageView height={24} width={24} style={[styles.closeBarItem.icon]} source={iconName} />
      </Button>
    );
  }
}

interface IBackBarItemProps {
  children?: React.ReactText;
  action: () => void;
}

export class BackBarItem extends Component<IBackBarItemProps> {
  public render() {
    return (
      <Button style={styles.backBarButton.default} onPress={this.props.action}>
        <View style={styles.backBarButton.content}>
          <ImageView style={styles.backBarButton.icon} source="icon-back" />
          <Text style={styles.backBarButton.label}>{this.props.children}</Text>
        </View>
      </Button>
    );
  }
}
