import React from 'react';
import styled from 'styled-components';

interface ISpacePreAllocationView {
  children?: React.ReactNode;
}

const StyledSpaceAllocationView = styled.div`
  display: flex;
  flex-direction: column;
  flex-grow: 1;
`;

export class SpacePreAllocationView extends React.Component<ISpacePreAllocationView> {
  private ref = React.createRef<HTMLDivElement | null>();

  public allocate(height: number) {
    if (this.ref.current) {
      this.minHeight = this.ref.current.offsetHeight + height + 'px';
    }
  }

  public reset = () => {
    this.minHeight = 'auto';
  };

  public render() {
    return (
      <StyledSpaceAllocationView ref={this.ref}>{this.props.children}</StyledSpaceAllocationView>
    );
  }

  private set minHeight(value: string) {
    const element = this.ref.current;
    if (element) {
      element.style.minHeight = value;
    }
  }
}
