import React from 'react';

interface ISpacePreAllocationView {
  children?: React.ReactNode;
}

export class SpacePreAllocationView extends React.Component<ISpacePreAllocationView> {
  private ref = React.createRef<HTMLDivElement>();

  public allocate(height: number) {
    if (this.ref.current) {
      this.minHeight = this.ref.current.offsetHeight + height + 'px';
    }
  }

  public reset = () => {
    this.minHeight = 'auto';
  };

  public render() {
    return <div ref={this.ref}>{this.props.children}</div>;
  }

  private set minHeight(value: string) {
    const element = this.ref.current;
    if (element) {
      element.style.minHeight = value;
    }
  }
}
