import React from 'react';
import styled from 'styled-components';

import { Scheduler } from '../../shared/scheduler';

const Container = styled.div({
  overflow: 'hidden',
});

const Text = styled.span({}, (props: { overflow: number; alignRight: boolean }) => ({
  display: 'inline-block',
  // Prevents Container from adding 2px below the text.
  verticalAlign: 'middle',
  whiteSpace: 'nowrap',
  willChange: props.overflow > 0 ? 'transform' : 'auto',
  transform: props.alignRight ? `translate3d(${-props.overflow}px, 0, 0)` : 'translate3d(0, 0, 0)',
  transition: `transform linear ${props.overflow * 80}ms`,
}));

interface IMarqueeProps {
  className?: string;
  children?: React.ReactNode;
}

interface IMarqueeState extends React.HTMLAttributes<HTMLSpanElement> {
  alignRight: boolean;
  // uniqueKey is used to force the Text component to remount to achieve the initial position of the
  // text without using a transition.
  uniqueKey: number;
}

export default class Marquee extends React.Component<IMarqueeProps, IMarqueeState> {
  public state = {
    alignRight: false,
    uniqueKey: 0,
  };

  private textRef = React.createRef<HTMLSpanElement>();
  private scheduler = new Scheduler();

  public componentDidMount() {
    this.startAnimationIfOverflow();
  }

  public componentDidUpdate(prevProps: IMarqueeProps) {
    if (this.props.children !== prevProps.children) {
      this.scheduler.cancel();
      this.setState(
        (state) => ({
          alignRight: false,
          uniqueKey: state.uniqueKey + 1,
        }),
        this.startAnimationIfOverflow,
      );
    }
  }

  public componentWillUnmount() {
    this.scheduler.cancel();
  }

  public render() {
    const { children, ...otherProps } = this.props;

    return (
      <Container>
        <Text
          key={this.state.uniqueKey}
          ref={this.textRef}
          overflow={this.calculateOverflow()}
          alignRight={this.state.alignRight}
          onTransitionEnd={this.scheduleToggleAlignRight}
          {...otherProps}>
          {children}
        </Text>
      </Container>
    );
  }

  private startAnimationIfOverflow = () => {
    if (this.calculateOverflow() > 0) {
      this.scheduleToggleAlignRight();
    }
  };

  private scheduleToggleAlignRight = () => {
    this.scheduler.schedule(() => {
      this.setState((state) => ({ alignRight: !state.alignRight }));
    }, 2000);
  };

  private calculateOverflow() {
    const textWidth = this.textRef.current?.offsetWidth ?? 0;
    const parentWidth = this.textRef.current?.parentElement?.offsetWidth ?? 0;
    return textWidth - parentWidth;
  }
}
