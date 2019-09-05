import * as React from 'react';
import * as ReactDOM from 'react-dom';
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
      backgroundColor: 'rgba(0, 0, 0, 0.2)',
      position: 'absolute',
      bottom: 0,
      left: 0,
      right: 0,
      height: 1,
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
  scopeBar: {
    container: Styles.createViewStyle({
      flexDirection: 'row',
      backgroundColor: colors.blue40,
      borderRadius: 13,
    }),
    item: {
      base: Styles.createButtonStyle({
        cursor: 'default',
        flex: 1,
        paddingHorizontal: 8,
        paddingVertical: 4,
      }),
      selected: Styles.createButtonStyle({
        backgroundColor: colors.blue,
      }),
      hover: Styles.createButtonStyle({
        backgroundColor: colors.blue40,
      }),
      label: Styles.createTextStyle({
        fontFamily: 'Open Sans',
        fontSize: 13,
        color: colors.white,
        textAlign: 'center',
      }),
    },
  },
  stickyContentHolder: Styles.createViewStyle({
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    backgroundColor: colors.darkBlue,
  }),
};

interface INavigationScrollContextValue {
  navigationContainer?: NavigationContainer;
  showsBarTitle: boolean;
  showsBarSeparator: boolean;
}

const NavigationScrollContext = React.createContext<INavigationScrollContextValue>({
  showsBarTitle: false,
  showsBarSeparator: false,
});

export class NavigationContainer extends Component {
  public state = {
    navigationContainer: this,
    showsBarTitle: false,
    showsBarSeparator: false,
  };

  private scrollEventListeners: Array<(event: IScrollEvent) => void> = [];

  public componentDidMount() {
    this.updateBarAppearance({ scrollLeft: 0, scrollTop: 0 });
  }

  public render() {
    return (
      <NavigationScrollContext.Provider value={this.state}>
        {this.props.children}
      </NavigationScrollContext.Provider>
    );
  }

  public onScroll = (event: IScrollEvent) => {
    this.notifyScrollEventListeners(event);
    this.updateBarAppearance(event);
  };

  public addScrollEventListener(fn: (event: IScrollEvent) => void) {
    const index = this.scrollEventListeners.indexOf(fn);
    if (index === -1) {
      this.scrollEventListeners.push(fn);
    }
  }

  public removeScrollEventListener(fn: (event: IScrollEvent) => void) {
    const index = this.scrollEventListeners.indexOf(fn);
    if (index !== -1) {
      this.scrollEventListeners.splice(index, 1);
    }
  }

  private notifyScrollEventListeners(event: IScrollEvent) {
    this.scrollEventListeners.forEach((listener) => listener(event));
  }

  private updateBarAppearance(event: IScrollEvent) {
    // detect if any of child elements provide a sticky context
    // in that case the navigation bar does not draw the separator line
    // since the sticky content is expected to include it.
    const hasSticky = React.Children.toArray(this.props.children).some((child) => {
      return React.isValidElement(child) && child.type === StickyContentContainer;
    });

    // that's where SettingsHeader.HeaderTitle intersects the navigation bar
    const showsBarSeparator = event.scrollTop > 11 && !hasSticky;

    // that's when SettingsHeader.HeaderTitle goes behind the navigation bar
    const showsBarTitle = event.scrollTop > 39;

    if (
      this.state.showsBarSeparator !== showsBarSeparator ||
      this.state.showsBarTitle !== showsBarTitle
    ) {
      this.setState({ showsBarSeparator, showsBarTitle });
    }
  }
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
    <PrivateNavigationScrollbars forwardedRef={ref} {...props}>
      {props.children}
    </PrivateNavigationScrollbars>
  );
});

interface IPrivateNavigationScrollbars extends INavigationScrollbarsProps {
  forwardedRef?: React.Ref<CustomScrollbars>;
}

class PrivateNavigationScrollbars extends Component<IPrivateNavigationScrollbars> {
  public static contextType = NavigationScrollContext;
  public context!: React.ContextType<typeof NavigationScrollContext>;

  public render() {
    return (
      <CustomScrollbars
        ref={this.props.forwardedRef}
        style={this.props.style}
        onScroll={this.onScroll}>
        {this.props.children}
      </CustomScrollbars>
    );
  }

  private onScroll = (scroll: IScrollEvent) => {
    if (this.context.navigationContainer) {
      this.context.navigationContainer.onScroll(scroll);
    }

    if (this.props.onScroll) {
      this.props.onScroll(scroll);
    }
  };
}

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

interface IStickyContentContext {
  container: HTMLDivElement | null;
  holder: React.Ref<View>;
  isSticky: boolean;
}

const StickyContentContext = React.createContext<IStickyContentContext>({
  container: null,
  holder: React.createRef<View>(),
  isSticky: false,
});

export class StickyContentContainer extends Component<{
  style: Types.StyleRuleSet<Types.ViewStyle>;
}> {
  public static contextType = NavigationScrollContext;
  public context!: React.ContextType<typeof NavigationScrollContext>;

  public state = {
    container: null,
    holder: React.createRef<View>(),
    isSticky: false,
  };

  public componentDidMount() {
    if (this.context.navigationContainer) {
      this.context.navigationContainer.addScrollEventListener(this.onScroll);
    }
  }

  public componentWillUnmount() {
    if (this.context.navigationContainer) {
      this.context.navigationContainer.removeScrollEventListener(this.onScroll);
    }
  }

  public render() {
    return (
      <div
        ref={this.onRef}
        style={{
          position: 'relative',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}>
        <View style={this.props.style}>
          <StickyContentContext.Provider value={this.state}>
            {this.props.children}
          </StickyContentContext.Provider>
        </View>
      </div>
    );
  }

  private onScroll = async (_scrollEvent: IScrollEvent) => {
    const holder = this.state.holder.current;

    if (holder) {
      let layout: Types.LayoutInfo;

      try {
        layout = await UserInterface.measureLayoutRelativeToAncestor(holder, this);
      } catch {
        // TODO: handle error
        return;
      }

      const isSticky = layout.y <= 0;

      if (this.state.isSticky !== isSticky) {
        this.setState({ isSticky });
      }
    }
  };

  private onRef = (ref: HTMLDivElement | null) => {
    this.setState({ container: ref });
  };
}

interface IScopeBarProps {
  defaultSelectedIndex: number;
  onChange?: (selectedIndex: number) => void;
  style?: Types.StyleRuleSet<Types.ViewStyle>;
  children: React.ReactNode;
}

interface IScopeBarState {
  selectedIndex: number;
}

export class ScopeBar extends Component<IScopeBarProps, IScopeBarState> {
  public static defaultProps: Partial<IScopeBarProps> = {
    defaultSelectedIndex: 0,
  };

  public state = {
    selectedIndex: 0,
  };

  constructor(props: IScopeBarProps) {
    super(props);

    this.state = {
      selectedIndex: props.defaultSelectedIndex,
    };
  }

  public render() {
    return (
      <View style={[styles.scopeBar.container, this.props.style]}>
        {React.Children.map(this.props.children, (child, index) => {
          if (React.isValidElement(child)) {
            return React.cloneElement(child, {
              ...(child.props || {}),
              selected: index === this.state.selectedIndex,
              onPress: this.makePressHandler(index),
            });
          } else {
            return undefined;
          }
        })}
      </View>
    );
  }

  public shouldComponentUpdate(nextProps: IScopeBarProps, nextState: IScopeBarState) {
    return (
      this.props.onChange !== nextProps.onChange ||
      this.props.style !== nextProps.style ||
      this.props.children !== nextProps.children ||
      this.state.selectedIndex !== nextState.selectedIndex
    );
  }

  private makePressHandler(index: number) {
    return () => {
      if (this.state.selectedIndex !== index) {
        this.setState({ selectedIndex: index }, () => {
          if (this.props.onChange) {
            this.props.onChange(index);
          }
        });
      }
    };
  }
}

interface IStickyContentHolderProps {
  style?: Types.ViewStyleRuleSet;
}

interface IStickyContentHolderState {
  contentHeight: number;
}

export class StickyContentHolder extends Component<
  IStickyContentHolderProps,
  IStickyContentHolderState
> {
  public state = {
    contentHeight: 0,
  };

  public render() {
    return (
      <StickyContentContext.Consumer>
        {(stickyContext) => {
          const contentStyle = stickyContext.isSticky ? styles.stickyContentHolder : undefined;
          const contentPlaceholderStyle = stickyContext.isSticky
            ? Styles.createViewStyle(
                {
                  height: this.state.contentHeight,
                },
                false,
              )
            : undefined;

          const children = (
            <View style={contentStyle} onLayout={this.onLayout}>
              {this.props.children}
              {stickyContext.isSticky ? <NavigationBarSeparator /> : undefined}
            </View>
          );

          return (
            <View style={this.props.style} ref={stickyContext.holder}>
              {stickyContext.isSticky && stickyContext.container ? (
                <React.Fragment>
                  <View style={contentPlaceholderStyle} />
                  {ReactDOM.createPortal(children, stickyContext.container)}
                </React.Fragment>
              ) : (
                children
              )}
            </View>
          );
        }}
      </StickyContentContext.Consumer>
    );
  }

  private onLayout = async (layout: Types.LayoutInfo) => {
    if (this.state.contentHeight !== layout.height) {
      this.setState({ contentHeight: layout.height });
    }
  };
}

interface IScopeBarItemProps {
  children?: React.ReactText;
  selected?: boolean;
  onPress?: () => void;
}

export class ScopeBarItem extends Component<IScopeBarItemProps> {
  public state = {
    isHovered: false,
  };

  public render() {
    const hoverStyle = this.props.selected
      ? styles.scopeBar.item.selected
      : this.state.isHovered
      ? styles.scopeBar.item.hover
      : undefined;

    return (
      <Button
        style={[styles.scopeBar.item.base, hoverStyle]}
        onHoverStart={this.onHoverStart}
        onHoverEnd={this.onHoverEnd}
        onPress={this.props.onPress}>
        <Text style={styles.scopeBar.item.label}>{this.props.children}</Text>
      </Button>
    );
  }

  private onHoverStart = () => {
    this.setState({ isHovered: true });
  };

  private onHoverEnd = () => {
    this.setState({ isHovered: false });
  };
}

function NavigationBarSeparator() {
  return <View style={styles.navigationBar.separator} />;
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
        <PrivateNavigationBar
          ref={ref}
          showsBarTitle={context.showsBarTitle}
          showsBarSeparator={context.showsBarSeparator}>
          {props.children}
        </PrivateNavigationBar>
      )}
    </NavigationScrollContext.Consumer>
  );
});

interface IPrivateNavigationBarProps {
  showsBarSeparator: boolean;
  showsBarTitle: boolean;
  children?: React.ReactNode;
}

interface IPrivateNavigationBarState {
  titleAdjustment: number;
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
  public state: IPrivateNavigationBarState = {
    titleAdjustment: 0,
  };

  private titleViewRef = React.createRef<PrivateTitleBarItem>();
  private measuringTextRef = React.createRef<Text>();

  public shouldComponentUpdate(
    nextProps: IPrivateNavigationBarProps,
    nextState: IPrivateNavigationBarState,
  ) {
    return (
      this.props.children !== nextProps.children ||
      this.props.showsBarSeparator !== nextProps.showsBarSeparator ||
      this.props.showsBarTitle !== nextProps.showsBarTitle ||
      this.state.titleAdjustment !== nextState.titleAdjustment
    );
  }

  public render() {
    return (
      <View style={[styles.navigationBar.default, this.getPlatformStyle()]}>
        <View style={styles.navigationBar.content} onLayout={this.onLayout}>
          <PrivateTitleBarItemContext.Provider
            value={{
              titleAdjustment: this.state.titleAdjustment,
              visible: this.props.showsBarTitle,
              titleRef: this.titleViewRef,
              measuringTextRef: this.measuringTextRef,
            }}>
            {this.props.children}
          </PrivateTitleBarItemContext.Provider>
        </View>
        {this.props.showsBarSeparator && <NavigationBarSeparator />}
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
