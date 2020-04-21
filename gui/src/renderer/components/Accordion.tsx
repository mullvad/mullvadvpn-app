import * as React from 'react';
import styled from 'styled-components';

interface IProps {
  expanded: boolean;
  animationDuration: number;
  children?: React.ReactNode;
}

interface IState {
  mountChildren: boolean;
  containerHeight: string;
}

const Container = styled.ul((props: { height: string; animationDuration: number }) => ({
  display: 'flex',
  height: props.height,
  overflow: 'hidden',
  transition: `height ${props.animationDuration}ms ease-in-out`,
}));

const Content = styled.li({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  height: 'fit-content',
});

export default class Accordion extends React.Component<IProps, IState> {
  private contentRef = React.createRef<HTMLLIElement>();

  public static defaultProps = {
    expanded: true,
    animationDuration: 350,
  };

  public state: IState = {
    mountChildren: this.props.expanded,
    containerHeight: this.props.expanded ? 'auto' : '0',
  };

  public componentDidUpdate(oldProps: IProps) {
    if (this.props.expanded !== oldProps.expanded) {
      // make sure the children are mounted first before expanding the accordion
      if (this.props.expanded && !this.state.mountChildren) {
        this.setState({ mountChildren: true }, () => {
          setTimeout(() => {
            this.setState({ containerHeight: this.getContentHeight() });
          });
        });
      } else if (this.props.expanded) {
        this.setState({ containerHeight: this.getContentHeight() });
      } else if (!this.props.expanded) {
        // First change height to height in px since it's not possible to transition to/from auto
        this.setState({ containerHeight: this.getContentHeight() }, () => {
          setTimeout(() => {
            this.setState({ containerHeight: '0px' });
          });
        });
      }
    }
  }

  public render() {
    return (
      <Container
        height={this.state.containerHeight}
        animationDuration={this.props.animationDuration}
        onTransitionEnd={this.onTransitionEnd}>
        <Content ref={this.contentRef}>{this.state.mountChildren && this.props.children}</Content>
      </Container>
    );
  }

  private getContentHeight(): string {
    return (this.contentRef.current?.getBoundingClientRect().height ?? 0) + 'px';
  }

  private onTransitionEnd = () => {
    if (this.props.expanded) {
      // Height auto enables the container to grow if the content changes size
      this.setState({ containerHeight: 'auto' });
    }
  };
}
