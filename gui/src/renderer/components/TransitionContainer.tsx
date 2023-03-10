import * as React from 'react';
import styled from 'styled-components';

import { ITransitionSpecification } from '../lib/history';
import { WillExit } from '../lib/will-exit';

interface ITransitioningViewProps {
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
  queuedItem?: ITransitionQueueItem;
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
    currentItem: TransitionContainer.makeItem(this.props),
  };

  private isCycling = false;
  private isTransitioning = false;

  private currentContentRef: React.MutableRefObject<HTMLDivElement | null> = React.createRef<HTMLDivElement>();
  private nextContentRef: React.MutableRefObject<HTMLDivElement | null> = React.createRef<HTMLDivElement>();
  // The item that should trigger the cycle to finish in onTransitionEnd
  private transitioningItemRef?: React.RefObject<HTMLDivElement>;

  public componentDidUpdate(prevProps: IProps) {
    if (this.props.children !== prevProps.children) {
      this.updateStateFromProps();
    }

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
    const willExit = this.state.queuedItem !== undefined || this.state.nextItem !== undefined;

    return (
      <StyledTransitionContainer disableUserInteraction={willExit}>
        {this.state.currentItem && (
          <WillExit key={this.state.currentItem.view.props.routePath} value={willExit}>
            <StyledTransitionContent
              ref={this.setCurrentContentRef}
              transition={this.state.currentItemStyle}
              onTransitionEnd={this.onTransitionEnd}>
              {this.state.currentItem.view}
            </StyledTransitionContent>
          </WillExit>
        )}

        {this.state.nextItem && (
          <WillExit key={this.state.nextItem.view.props.routePath} value={false}>
            <StyledTransitionContent
              ref={this.setNextContentRef}
              transition={this.state.nextItemStyle}
              onTransitionEnd={this.onTransitionEnd}>
              {this.state.nextItem.view}
            </StyledTransitionContent>
          </WillExit>
        )}
      </StyledTransitionContainer>
    );
  }

  private setCurrentContentRef = (element: HTMLDivElement) => {
    this.currentContentRef.current?.removeEventListener('transitionstart', this.onTransitionStart);
    this.currentContentRef.current = element;
    this.currentContentRef.current?.addEventListener('transitionstart', this.onTransitionStart);
  };

  private setNextContentRef = (element: HTMLDivElement) => {
    this.nextContentRef.current?.removeEventListener('transitionstart', this.onTransitionStart);
    this.nextContentRef.current = element;
    this.nextContentRef.current?.addEventListener('transitionstart', this.onTransitionStart);
  };

  private updateStateFromProps() {
    const candidate = this.props.children;

    if (candidate && this.state.currentItem) {
      // Update currentItem, nextItem, queuedItem depending on which the candidate matches.
      if (
        !this.isTransitioning &&
        this.state.currentItem.view.props.routePath === candidate.props.routePath
      ) {
        // There's no transition in progress and the newest candidate has the same path as the
        // current. In this sitation the app should just remain in the same view.
        this.setState(
          {
            currentItem: TransitionContainer.makeItem(this.props),
            nextItem: undefined,
            queuedItem: undefined,
            currentItemStyle: undefined,
            nextItemStyle: undefined,
            currentItemTransition: undefined,
            nextItemTransition: undefined,
          },
          () => (this.isCycling = false),
        );
      } else if (!this.isTransitioning && this.state.nextItem) {
        // There's no transition in progress but there is a next item. Abort the transition and add
        // the candidate to the queue. The app shouldn't start a transition if there is another view
        // to queue.
        this.setState(
          {
            nextItem: undefined,
            queuedItem: TransitionContainer.makeItem(this.props),
            currentItemStyle: undefined,
            nextItemStyle: undefined,
            currentItemTransition: undefined,
            nextItemTransition: undefined,
          },
          () => (this.isCycling = false),
        );
      } else if (this.state.nextItem?.view.props.routePath === candidate.props.routePath) {
        // There's an update to the item that is currently being transitioned to. Update that item
        // and continue the transition.
        this.setState({
          nextItem: TransitionContainer.makeItem(this.props),
          queuedItem: undefined,
        });
      } else {
        // If none of the above, initiate a transition to the new item.
        this.setState({ queuedItem: TransitionContainer.makeItem(this.props) });
      }
    } else if (candidate) {
      // Child is set as current item if there's no item already.
      this.setState({ currentItem: TransitionContainer.makeItem(this.props) });
    }
  }

  private onTransitionStart = (event: TransitionEvent) => {
    if (
      this.isCycling &&
      !this.isTransitioning &&
      event.target === this.transitioningItemRef?.current
    ) {
      this.isTransitioning = true;
    }
  };

  private onTransitionEnd = (event: React.TransitionEvent<HTMLDivElement>) => {
    if (this.isCycling && event.target === this.transitioningItemRef?.current) {
      this.isTransitioning = false;
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
    if (this.state.queuedItem) {
      const transition = this.state.queuedItem.transition;

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
      nextItem: state.queuedItem,
      queuedItem: undefined,
      currentItemStyle: { x: 0, y: 0, inFront: false },
      nextItemStyle: { x: 0, y: 100, inFront: true },
      currentItemTransition: { duration },
      nextItemTransition: { y: 0, duration },
    }));
  }

  private slideDown(duration: number) {
    this.transitioningItemRef = this.currentContentRef;
    this.setState((state) => ({
      nextItem: state.queuedItem,
      queuedItem: undefined,
      currentItemStyle: { x: 0, y: 0, inFront: true },
      nextItemStyle: { x: 0, y: 0, inFront: false },
      currentItemTransition: { y: 100, duration },
      nextItemTransition: { duration },
    }));
  }

  private push(duration: number) {
    this.transitioningItemRef = this.nextContentRef;
    this.setState((state) => ({
      nextItem: state.queuedItem,
      queuedItem: undefined,
      currentItemStyle: { x: 0, y: 0, inFront: false },
      nextItemStyle: { x: 100, y: 0, inFront: true },
      currentItemTransition: { x: -50, duration },
      nextItemTransition: { x: 0, duration },
    }));
  }

  private pop(duration: number) {
    this.transitioningItemRef = this.currentContentRef;
    this.setState((state) => ({
      nextItem: state.queuedItem,
      queuedItem: undefined,
      currentItemStyle: { x: 0, y: 0, inFront: true },
      nextItemStyle: { x: -50, y: 0, inFront: false },
      currentItemTransition: { x: 100, duration },
      nextItemTransition: { x: 0, duration },
    }));
  }

  private replace(completion: () => void) {
    this.setState(
      (state) => ({
        currentItem: state.queuedItem,
        nextItem: undefined,
        queuedItem: undefined,
        currentItemStyle: { x: 0, y: 0, inFront: false, duration: 0 },
        nextItemStyle: { x: 0, y: 0, inFront: true, duration: 0 },
        currentItemTransition: undefined,
        nextItemTransition: undefined,
      }),
      completion,
    );
  }
}
