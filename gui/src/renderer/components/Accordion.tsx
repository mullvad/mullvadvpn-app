import * as React from 'react';
import styled from 'styled-components';

interface IProps {
  expanded: boolean;
  animationDuration: number;
  children?: React.ReactNode;
  onWillExpand?: (contentHeight: number) => void;
  onTransitionEnd?: () => void;
}

interface IState {
  mountChildren: boolean;
  containerHeight: string;
}

const Container = styled.div((props: { height: string; animationDuration: number }) => ({
  display: 'flex',
  height: props.height,
  overflow: 'hidden',
  transition: `height ${props.animationDuration}ms ease-in-out`,
}));

const Content = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  height: 'fit-content',
  width: '100%',
});

export default class Accordion extends React.Component<IProps, IState> {
  public static defaultProps = {
    expanded: true,
    animationDuration: 350,
  };

  public state: IState = {
    mountChildren: this.props.expanded,
    containerHeight: this.props.expanded ? 'auto' : '0',
  };

  private containerRef = React.createRef<HTMLDivElement>();
  private contentRef = React.createRef<HTMLDivElement>();

  public componentDidUpdate(oldProps: IProps) {
    if (this.props.expanded && !oldProps.expanded) {
      this.expand();
    } else if (!this.props.expanded && oldProps.expanded) {
      this.collapse();
    }
  }

  public render() {
    return (
      <Container
        ref={this.containerRef}
        height={this.state.containerHeight}
        animationDuration={this.props.animationDuration}
        onTransitionEnd={this.onTransitionEnd}>
        <Content ref={this.contentRef}>{this.state.mountChildren && this.props.children}</Content>
      </Container>
    );
  }

  private expand() {
    // Make sure the children are mounted first before expanding the accordion
    this.mountChildren(() => {
      this.onWillExpand();
      this.setState({ containerHeight: this.getContentHeightWithUnit() });
    });
  }

  private mountChildren(childrenDidMount: () => void) {
    if (!this.state.mountChildren) {
      this.setState({ mountChildren: true }, childrenDidMount);
    } else {
      childrenDidMount();
    }
  }

  private collapse() {
    // First change height to height in px since it's not possible to transition to/from auto
    this.setState({ containerHeight: this.getContentHeightWithUnit() }, () => {
      // Make sure new height has been applied
      // eslint-disable-next-line @typescript-eslint/no-unused-expressions
      this.containerRef.current?.offsetHeight;
      this.setState({ containerHeight: '0' });
    });
  }

  private getContentHeightWithUnit(): string {
    return (this.getContentHeight() ?? 0) + 'px';
  }

  private getContentHeight(): number | undefined {
    return this.contentRef.current?.offsetHeight;
  }

  private onWillExpand() {
    const contentHeight = this.getContentHeight();
    if (contentHeight) {
      this.props.onWillExpand?.(contentHeight);
    }
  }

  private onTransitionEnd = () => {
    this.props.onTransitionEnd?.();
    if (this.props.expanded) {
      // Height auto enables the container to grow if the content changes size
      this.setState({ containerHeight: 'auto' });
    }
  };
}
