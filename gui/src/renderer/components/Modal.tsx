import * as React from 'react';

export class ModalContent extends React.Component {
  public render() {
    return (
      <div
        style={{
          position: 'absolute',
          display: 'flex',
          flexDirection: 'column',
          flex: 1,
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
        }}>
        {this.props.children}
      </div>
    );
  }
}

export class ModalAlert extends React.Component {
  public render() {
    return (
      <div
        style={{
          backgroundColor: 'rgba(0,0,0,0.5)',
          position: 'absolute',
          display: 'flex',
          flexDirection: 'column',
          flex: 1,
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
        }}>
        {this.props.children}
      </div>
    );
  }
}

interface IModalContainerProps {
  children?: React.ReactNode;
}

export class ModalContainer extends React.Component<IModalContainerProps> {
  public render() {
    return <div style={{ position: 'relative', flex: 1 }}>{this.props.children}</div>;
  }
}
