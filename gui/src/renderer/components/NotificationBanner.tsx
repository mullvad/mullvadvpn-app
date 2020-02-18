import * as React from 'react';
import { Animated, Button, Component, Styles, Text, Types, UserInterface, View } from 'reactxp';
import { colors } from '../../config.json';
import { BlockingButton } from './AppButton';
import ImageView from './ImageView';

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

interface INotificationTitleProps {
  children?: React.ReactNode;
}

export class NotificationTitle extends Component<INotificationTitleProps> {
  public render() {
    return <Text style={styles.title}>{this.props.children}</Text>;
  }
}

interface INotificationSubtitleProps {
  children?: React.ReactNode;
}

export class NotificationSubtitle extends Component<INotificationSubtitleProps> {
  public render() {
    return React.Children.count(this.props.children) > 0 ? (
      <Text style={styles.subtitle}>{this.props.children}</Text>
    ) : null;
  }
}

interface INotifcationOpenLinkActionProps {
  onPress: () => Promise<void>;
  children?: React.ReactNode;
}

export class NotificationOpenLinkAction extends Component<INotifcationOpenLinkActionProps> {
  public state = {
    hovered: false,
  };

  public render() {
    return (
      <BlockingButton onPress={this.props.onPress}>
        <Button
          style={styles.actionButton}
          onHoverStart={this.onHoverStart}
          onHoverEnd={this.onHoverEnd}>
          <ImageView
            height={12}
            width={12}
            tintColor={this.state.hovered ? colors.white80 : colors.white60}
            source="icon-extLink"
          />
        </Button>
      </BlockingButton>
    );
  }

  private onHoverStart = () => {
    this.setState({ hovered: true });
  };

  private onHoverEnd = () => {
    this.setState({ hovered: false });
  };
}

interface INotificationContentProps {
  children?: React.ReactNode;
}

export class NotificationContent extends Component<INotificationContentProps> {
  public render() {
    return <View style={styles.textContainer}>{this.props.children}</View>;
  }
}

interface INotificationActionsProps {
  children?: React.ReactNode;
}

export class NotificationActions extends Component<INotificationActionsProps> {
  public render() {
    return <View style={styles.actionContainer}>{this.props.children}</View>;
  }
}

interface INotificationIndicatorProps {
  type: 'success' | 'warning' | 'error';
  children?: React.ReactNode;
}

export class NotificationIndicator extends Component<INotificationIndicatorProps> {
  public render() {
    return <View style={[styles.indicator.base, styles.indicator[this.props.type]]} />;
  }
}

interface INotificationBannerProps {
  children: React.ReactNode; // Array<NotificationContent | NotificationActions>,
  style?: Types.ViewStyleRuleSet;
  visible: boolean;
  animationDuration: number;
}

interface INotificationBannerState {
  contentPinnedToBottom: boolean;
}

export class NotificationBanner extends Component<
  INotificationBannerProps,
  INotificationBannerState
> {
  public static defaultProps = {
    animationDuration: 350,
  };

  public state = {
    contentPinnedToBottom: false,
  };

  private containerRef = React.createRef<Animated.View>();
  private contentHeight = 0;
  private heightValue = Animated.createValue(0);
  private animationStyle: Types.AnimatedViewStyleRuleSet;
  private animation?: Types.Animated.CompositeAnimation;
  private didFinishFirstLayoutPass = false;

  constructor(props: INotificationBannerProps) {
    super(props);

    this.animationStyle = Styles.createAnimatedViewStyle({
      height: this.heightValue,
    });
  }

  public shouldComponentUpdate(
    nextProps: INotificationBannerProps,
    nextState: INotificationBannerState,
  ) {
    return (
      this.props.children !== nextProps.children ||
      this.props.visible !== nextProps.visible ||
      this.state.contentPinnedToBottom !== nextState.contentPinnedToBottom
    );
  }

  public componentDidUpdate(prevProps: INotificationBannerProps) {
    if (prevProps.visible !== this.props.visible) {
      // enable drawer-like animation when changing banner's visibility
      this.setState({ contentPinnedToBottom: true }, () => {
        this.animateHeightChanges();
      });
    }
  }

  public componentWillUnmount() {
    if (this.animation) {
      this.animation.stop();
    }
  }

  public render() {
    return (
      <Animated.View
        style={[
          styles.collapsible,
          this.state.contentPinnedToBottom ? styles.drawer : undefined,
          this.animationStyle,
          this.props.style,
        ]}
        ref={this.containerRef}>
        <View onLayout={this.onLayout}>
          <View style={styles.container}>{this.props.children}</View>
        </View>
      </Animated.View>
    );
  }

  private onLayout = ({ height }: Types.ViewOnLayoutEvent) => {
    const oldHeight = this.contentHeight;
    this.contentHeight = height;

    // The first layout pass should not be animated because this would cause the initially visible
    // notification banner to slide down each time the component is mounted.
    if (this.didFinishFirstLayoutPass) {
      if (oldHeight !== height) {
        this.animateHeightChanges();
      }
    } else {
      this.didFinishFirstLayoutPass = true;
      if (this.props.visible) {
        this.stopAnimation();
        this.heightValue.setValue(height);
      }
    }
  };

  private async animateHeightChanges() {
    const containerView = this.containerRef.current;
    if (!containerView) {
      return;
    }

    this.stopAnimation();

    // calculate the animation duration based on travel distance
    const layout = await UserInterface.measureLayoutRelativeToWindow(containerView);
    const toValue = this.props.visible ? this.contentHeight : 0;
    const multiplier = Math.abs(toValue - layout.height) / Math.max(1, this.contentHeight);
    const duration = Math.ceil(this.props.animationDuration * multiplier);

    const animation = Animated.timing(this.heightValue, {
      toValue,
      easing: Animated.Easing.InOut(),
      duration,
      useNativeDriver: true,
    });

    this.animation = animation;

    animation.start(({ finished }) => {
      if (finished) {
        // disable drawer-like animations for content updates when the banner is visible
        this.setState({ contentPinnedToBottom: false });
      }
    });
  }

  private stopAnimation() {
    if (this.animation) {
      this.animation.stop();
      this.animation = undefined;
    }
  }
}
