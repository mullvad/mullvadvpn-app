import * as React from 'react';
import styled from 'styled-components';

import { ITransitionSpecification } from '../lib/history';
import { WillExit } from '../lib/will-exit';

interface ITransitioningViewProps {
  viewId: string;
  routePath: string;
  children?: React.ReactNode;
}

type TransitioningView = React.ReactElement<ITransitioningViewProps>;

interface ITransitionQueueItem {
  view: TransitioningView;
  transition: ITransitionSpecification;
}

interface IProps extends ITransitionSpecification {
  children: TransitioningView;
  onTransitionEnd: () => void;
}

interface IItemStyle {
  // x and y are percentages
  x: number;
  y: number;
  inFront: boolean;
  duration?: number;
}

interface IState {
  currentItem?: ITransitionQueueItem;
  nextItem?: ITransitionQueueItem;
  itemQueue: ITransitionQueueItem[];
  currentItemStyle?: IItemStyle;
  nextItemStyle?: IItemStyle;
  currentItemTransition?: Partial<IItemStyle>;
  nextItemTransition?: Partial<IItemStyle>;
}

export const StyledTransitionContainer = styled.div(
  {},
  (props: { disableUserInteraction: boolean }) => ({
    flex: 1,
    pointerEvents: props.disableUserInteraction ? 'none' : undefined,
  }),
);

export const StyledTransitionContent = styled.div.attrs({ 'data-testid': 'transition-content' })(
  {},
  (props: { transition?: IItemStyle }) => {
    const x = `${props.transition?.x ?? 0}%`;
    const y = `${props.transition?.y ?? 0}%`;
    const duration = props.transition?.duration ?? 450;

    return {
      display: 'flex',
      flexDirection: 'column',
      position: 'absolute',
      left: 0,
      right: 0,
      top: 0,
      bottom: 0,
      zIndex: props.transition?.inFront ? 1 : 0,
      willChange: 'transform',
      transform: `translate3d(${x}, ${y}, 0)`,
      transition: `transform ${duration}ms ease-in-out`,
    };
  },
);

export const StyledTransitionView = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  height: '100%',
  width: '100%',
});

export class TransitionView extends React.Component<ITransitioningViewProps> {
  public render() {
    return (
      <StyledTransitionView data-testid={this.props.routePath}>
        {this.props.children}
      </StyledTransitionView>
    );
  }
}

export default class TransitionContainer extends React.Component<IProps, IState> {
  public state: IState = {
    itemQueue: [],
    currentItem: TransitionContainer.makeItem(this.props),
  };

  private isCycling = false;

  private currentContentRef = React.createRef<HTMLDivElement>();
  private nextContentRef = React.createRef<HTMLDivElement>();
  // The item that should trigger the cycle to finish in onTransitionEnd
  private transitioningItemRef?: React.RefObject<HTMLDivElement>;

  public static getDerivedStateFromProps(props: IProps, state: IState) {
    const candidate = props.children;

    if (candidate && state.currentItem) {
      // Synchronize updates to the last added child. Although the queue doesn't change, the child
      // itself might need to change. That's why the queue-/next item is replaced by it again after
      // calling `makeItem`.
      const itemQueueCount = state.itemQueue.length;
      const lastItemInQueue = itemQueueCount > 0 ? state.itemQueue[itemQueueCount - 1] : undefined;

      if (lastItemInQueue && lastItemInQueue.view.props.viewId === candidate.props.viewId) {
        // Child is last item in queue. No change to the queue needed.
        return {
          itemQueue: [...state.itemQueue.slice(0, -1), TransitionContainer.makeItem(props)],
        };
      } else if (
        itemQueueCount === 0 &&
        state.nextItem &&
        state.nextItem.view.props.viewId === candidate.props.viewId
      ) {
        // Child is next item, no change to the queue needed.
        return { nextItem: TransitionContainer.makeItem(props) };
      } else if (
        itemQueueCount === 0 &&
        !state.nextItem &&
        state.currentItem.view.props.viewId === candidate.props.viewId
      ) {
        // Child is current item and there's no new child, no change to the queue needed.
        return { currentItem: TransitionContainer.makeItem(props) };
      } else {
        // Child is a new item and is added to the queue.
        return { itemQueue: [...state.itemQueue, TransitionContainer.makeItem(props)] };
      }
    } else if (candidate && !state.currentItem) {
      // Child is set as current item if there's no item already.
      return { currentItem: TransitionContainer.makeItem(props) };
    } else {
      return null;
    }
  }

  public componentDidUpdate() {
    if (
      this.state.currentItemStyle &&
      this.state.currentItemTransition &&
      this.state.nextItemStyle &&
      this.state.nextItemTransition
    ) {
      // Force browser reflow before starting transition. Without this animations won't run since
      // the next view content hasn't been painted yet. It will just appear without a transition.
      void this.nextContentRef.current?.offsetHeight;

      // Start transition
      this.setState((state) => ({
        currentItemStyle: Object.assign({}, state.currentItemStyle, state.currentItemTransition),
        nextItemStyle: Object.assign({}, state.nextItemStyle, state.nextItemTransition),
        currentItemTransition: undefined,
        nextItemTransition: undefined,
      }));
    } else {
      this.cycle();
    }
  }

  public render() {
    const willExit = this.state.itemQueue.length > 0 || this.state.nextItem !== undefined;

    return (
      <StyledTransitionContainer disableUserInteraction={willExit}>
        {this.state.currentItem && (
          <WillExit key={this.state.currentItem.view.props.viewId} value={willExit}>
            <StyledTransitionContent
              ref={this.currentContentRef}
              transition={this.state.currentItemStyle}
              onTransitionEnd={this.onTransitionEnd}>
              {this.state.currentItem.view}
            </StyledTransitionContent>
          </WillExit>
        )}

        {this.state.nextItem && (
          <WillExit key={this.state.nextItem.view.props.viewId} value={false}>
            <StyledTransitionContent
              ref={this.nextContentRef}
              transition={this.state.nextItemStyle}
              onTransitionEnd={this.onTransitionEnd}>
              {this.state.nextItem.view}
            </StyledTransitionContent>
          </WillExit>
        )}
      </StyledTransitionContainer>
    );
  }

  private onTransitionEnd = (event: React.TransitionEvent<HTMLDivElement>) => {
    if (this.isCycling && event.target === this.transitioningItemRef?.current) {
      this.transitioningItemRef = undefined;
      this.makeNextItemCurrent(() => {
        this.onFinishCycle();
      });
    }
  };

  private cycle() {
    if (!this.isCycling) {
      this.isCycling = true;
      this.cycleUnguarded();
    }
  }

  private onFinishCycle() {
    this.props.onTransitionEnd();
    this.cycleUnguarded();
  }

  private cycleUnguarded = () => {
    const itemQueue = this.state.itemQueue;

    if (itemQueue.length > 0) {
      const nextItem = itemQueue[0];
      const transition = nextItem.transition;

      switch (transition.name) {
        case 'slide-up':
          this.slideUp(transition.duration);
          break;

        case 'slide-down':
          this.slideDown(transition.duration);
          break;

        case 'push':
          this.push(transition.duration);
          break;

        case 'pop':
          this.pop(transition.duration);
          break;

        default:
          this.replace(() => this.onFinishCycle);
          break;
      }
    } else {
      this.isCycling = false;
    }
  };

  private static makeItem(props: IProps): ITransitionQueueItem {
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
        currentItemStyle: undefined,
        nextItemStyle: undefined,
        currentItemTransition: undefined,
        nextItemTransition: undefined,
      }),
      completion,
    );
  }

  private slideUp(duration: number) {
    this.transitioningItemRef = this.nextContentRef;
    this.setState((state) => ({
      nextItem: state.itemQueue[0],
      itemQueue: state.itemQueue.slice(1),
      currentItemStyle: { x: 0, y: 0, inFront: false },
      nextItemStyle: { x: 0, y: 100, inFront: true },
      currentItemTransition: { duration },
      nextItemTransition: { y: 0, duration },
    }));
  }

  private slideDown(duration: number) {
    this.transitioningItemRef = this.currentContentRef;
    this.setState((state) => ({
      nextItem: state.itemQueue[0],
      itemQueue: state.itemQueue.slice(1),
      currentItemStyle: { x: 0, y: 0, inFront: true },
      nextItemStyle: { x: 0, y: 0, inFront: false },
      currentItemTransition: { y: 100, duration },
      nextItemTransition: { duration },
    }));
  }

  private push(duration: number) {
    this.transitioningItemRef = this.nextContentRef;
    this.setState((state) => ({
      nextItem: state.itemQueue[0],
      itemQueue: state.itemQueue.slice(1),
      currentItemStyle: { x: 0, y: 0, inFront: false },
      nextItemStyle: { x: 100, y: 0, inFront: true },
      currentItemTransition: { x: -50, duration },
      nextItemTransition: { x: 0, duration },
    }));
  }

  private pop(duration: number) {
    this.transitioningItemRef = this.currentContentRef;
    this.setState((state) => ({
      nextItem: state.itemQueue[0],
      itemQueue: state.itemQueue.slice(1),
      currentItemStyle: { x: 0, y: 0, inFront: true },
      nextItemStyle: { x: -50, y: 0, inFront: false },
      currentItemTransition: { x: 100, duration },
      nextItemTransition: { x: 0, duration },
    }));
  }

  private replace(completion: () => void) {
    this.setState(
      (state) => ({
        currentItem: state.itemQueue[0],
        nextItem: undefined,
        itemQueue: state.itemQueue.slice(1),
        currentItemStyle: { x: 0, y: 0, inFront: false, duration: 0 },
        nextItemStyle: { x: 0, y: 0, inFront: true, duration: 0 },
        currentItemTransition: undefined,
        nextItemTransition: undefined,
      }),
      completion,
    );
  }
}
