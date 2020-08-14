import * as React from 'react';
import { Animated, Component, Styles, Types, View } from 'reactxp';
import { ITransitionGroupProps } from '../transitions';

interface ITransitioningViewProps {
  viewId: string;
}

type TransitioningView = React.ReactElement<ITransitioningViewProps>;

interface ITransitionQueueItem {
  view: TransitioningView;
  transition: ITransitionGroupProps;
}

interface IProps extends ITransitionGroupProps {
  children: TransitioningView;
}

interface IState {
  currentItem?: ITransitionQueueItem;
  nextItem?: ITransitionQueueItem;
  itemQueue: ITransitionQueueItem[];
  currentItemStyle?: Array<Types.StyleRuleSet<Types.AnimatedViewStyle | Types.ViewStyle>>;
  nextItemStyle?: Array<Types.StyleRuleSet<Types.AnimatedViewStyle | Types.ViewStyle>>;
}

const styles = {
  animatedContainer: Styles.createViewStyle({
    position: 'absolute',
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
  }),
  transitionView: Styles.createViewStyle({
    flex: 1,
  }),
  blockUserInteraction: Styles.createViewStyle({
    // @ts-ignore
    pointerEvents: 'none',
  }),
  transitionContainer: Styles.createViewStyle({
    flex: 1,
  }),
  orderFront: Styles.createViewStyle({
    // @ts-ignore
    zIndex: 1,
  }),
  orderBack: Styles.createViewStyle({
    // @ts-ignore
    zIndex: 0,
  }),
};

export class TransitionView extends Component<ITransitioningViewProps> {
  public render() {
    return <View style={styles.transitionView}>{this.props.children}</View>;
  }
}

export default class TransitionContainer extends Component<IProps, IState> {
  public state: IState = {
    itemQueue: [],
  };

  private containerSize = { width: 0, height: 0 };

  private animation?: Types.Animated.CompositeAnimation;
  private isCycling = false;

  private slideValueA = Animated.createValue(0);
  private slideAnimationStyleA = Styles.createAnimatedViewStyle({
    transform: [{ translateY: this.slideValueA }],
  });

  private slideValueB = Animated.createValue(0);
  private slideAnimationStyleB = Styles.createAnimatedViewStyle({
    transform: [{ translateY: this.slideValueB }],
  });

  private pushValueA = Animated.createValue(0);
  private pushStyleA = Styles.createAnimatedViewStyle({
    transform: [{ translateX: this.pushValueA }],
  });

  private pushValueB = Animated.createValue(0);
  private pushStyleB = Styles.createAnimatedViewStyle({
    transform: [{ translateX: this.pushValueB }],
  });

  constructor(props: IProps) {
    super(props);

    this.state.currentItem = this.makeItem(props);
  }

  public UNSAFE_componentWillReceiveProps(nextProps: IProps) {
    const candidate = nextProps.children;

    if (candidate && this.state.currentItem) {
      // synchronize updates to the last added child.
      const itemQueueCount = this.state.itemQueue.length;
      const lastItemInQueue =
        itemQueueCount > 0 ? this.state.itemQueue[itemQueueCount - 1] : undefined;

      if (lastItemInQueue && lastItemInQueue.view.props.viewId === candidate.props.viewId) {
        this.setState({
          itemQueue: [...this.state.itemQueue.slice(0, -1), this.makeItem(nextProps)],
        });
      } else if (
        itemQueueCount === 0 &&
        this.state.nextItem &&
        this.state.nextItem.view.props.viewId === candidate.props.viewId
      ) {
        this.setState({
          nextItem: this.makeItem(nextProps),
        });
      } else if (
        itemQueueCount === 0 &&
        !this.state.nextItem &&
        this.state.currentItem.view.props.viewId === candidate.props.viewId
      ) {
        this.setState({
          currentItem: this.makeItem(nextProps),
        });
      } else {
        // add new item
        this.setState({
          itemQueue: [...this.state.itemQueue, this.makeItem(nextProps)],
        });
      }
    } else if (candidate && !this.state.currentItem) {
      this.setState({ currentItem: this.makeItem(nextProps) });
    }
  }

  public componentDidUpdate() {
    this.cycle();
  }

  public componentWillUnmount() {
    if (this.animation) {
      this.animation.stop();
    }
  }

  public render() {
    const disableUserInteraction =
      this.state.itemQueue.length > 0 || this.state.nextItem ? true : false;

    return (
      <View
        style={[
          styles.transitionContainer,
          disableUserInteraction ? styles.blockUserInteraction : undefined,
        ]}
        onLayout={this.onLayout}>
        {this.state.currentItem && (
          <Animated.View
            key={this.state.currentItem.view.props.viewId}
            style={[styles.animatedContainer, this.state.currentItemStyle]}>
            {this.state.currentItem.view}
          </Animated.View>
        )}

        {this.state.nextItem && (
          <Animated.View
            key={this.state.nextItem.view.props.viewId}
            style={[styles.animatedContainer, this.state.nextItemStyle]}>
            {this.state.nextItem.view}
          </Animated.View>
        )}
      </View>
    );
  }

  private onLayout = (event: Types.ViewOnLayoutEvent) => {
    this.containerSize = { width: event.width, height: event.height };
  };

  private cycle() {
    if (!this.isCycling) {
      this.isCycling = true;
      this.cycleUnguarded(() => {
        this.isCycling = false;
      });
    }
  }

  private cycleUnguarded(onFinish: () => void) {
    const itemQueue = this.state.itemQueue;

    const continueCycling = () => {
      this.makeNextItemCurrent(() => {
        this.cycleUnguarded(onFinish);
      });
    };

    if (itemQueue.length > 0) {
      const nextItem = itemQueue[0];
      const transition = nextItem.transition;

      switch (transition.name) {
        case 'slide-up':
          this.slideUp(transition.duration, continueCycling);
          break;

        case 'slide-down':
          this.slideDown(transition.duration, continueCycling);
          break;

        case 'push':
          this.push(transition.duration, continueCycling);
          break;

        case 'pop':
          this.pop(transition.duration, continueCycling);
          break;

        default:
          this.replace(() => {
            this.cycleUnguarded(onFinish);
          });
          break;
      }
    } else {
      this.animation = undefined;
      onFinish();
    }
  }

  private makeItem(props: IProps): ITransitionQueueItem {
    return {
      transition: {
        name: props.name,
        duration: props.duration,
      },
      view: React.cloneElement(props.children),
    };
  }

  private makeNextItemCurrent(completion: () => void) {
    this.setState(
      (state) => ({
        currentItem: state.nextItem,
        nextItem: undefined,
        currentItemStyle: [],
        nextItemStyle: [],
      }),
      completion,
    );
  }

  private slideUp(duration: number, completion: Types.Animated.EndCallback) {
    this.slideValueA.setValue(0);
    this.slideValueB.setValue(this.containerSize.height);

    this.setState(
      (state) => ({
        nextItem: state.itemQueue[0],
        itemQueue: state.itemQueue.slice(1),
        currentItemStyle: [this.slideAnimationStyleA, styles.orderBack],
        nextItemStyle: [this.slideAnimationStyleB, styles.orderFront],
      }),
      () => {
        const animation = Animated.timing(this.slideValueB, {
          toValue: 0,
          easing: Animated.Easing.InOut(),
          duration,
        });

        animation.start(completion);
        this.animation = animation;
      },
    );
  }

  private slideDown(duration: number, completion: Types.Animated.EndCallback) {
    this.slideValueA.setValue(0);
    this.slideValueB.setValue(0);

    this.setState(
      (state) => ({
        nextItem: state.itemQueue[0],
        itemQueue: state.itemQueue.slice(1),
        currentItemStyle: [this.slideAnimationStyleA, styles.orderFront],
        nextItemStyle: [this.slideAnimationStyleB, styles.orderBack],
      }),
      () => {
        const animation = Animated.timing(this.slideValueA, {
          toValue: this.containerSize.height,
          easing: Animated.Easing.InOut(),
          duration,
        });

        animation.start(completion);
        this.animation = animation;
      },
    );
  }

  private push(duration: number, completion: Types.Animated.EndCallback) {
    this.pushValueA.setValue(0);
    this.pushValueB.setValue(this.containerSize.width);

    this.setState(
      (state) => ({
        nextItem: state.itemQueue[0],
        itemQueue: state.itemQueue.slice(1),
        currentItemStyle: [this.pushStyleA, styles.orderBack],
        nextItemStyle: [this.pushStyleB, styles.orderFront],
      }),
      () => {
        const animation = Animated.parallel([
          Animated.timing(this.pushValueA, {
            toValue: -this.containerSize.width * 0.5,
            easing: Animated.Easing.InOut(),
            duration,
          }),
          Animated.timing(this.pushValueB, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration,
          }),
        ]);

        animation.start(completion);
        this.animation = animation;
      },
    );
  }

  private pop(duration: number, completion: Types.Animated.EndCallback) {
    this.pushValueA.setValue(-this.containerSize.width * 0.5);
    this.pushValueB.setValue(0);

    this.setState(
      (state) => ({
        nextItem: state.itemQueue[0],
        itemQueue: state.itemQueue.slice(1),
        currentItemStyle: [this.pushStyleB, styles.orderFront],
        nextItemStyle: [this.pushStyleA, styles.orderBack],
      }),
      () => {
        const animation = Animated.parallel([
          Animated.timing(this.pushValueA, {
            toValue: 0,
            easing: Animated.Easing.InOut(),
            duration,
          }),
          Animated.timing(this.pushValueB, {
            toValue: this.containerSize.width,
            easing: Animated.Easing.InOut(),
            duration,
          }),
        ]);

        animation.start(completion);
        this.animation = animation;
      },
    );
  }

  private replace(completion: () => void) {
    this.setState(
      (state) => ({
        currentItem: state.itemQueue[0],
        nextItem: undefined,
        itemQueue: state.itemQueue.slice(1),
        currentItemStyle: [],
        nextItemStyle: [],
      }),
      completion,
    );
  }
}
