// @flow
import * as React from 'react';
import HeaderBar from './HeaderBar';

import type { HeaderBarProps } from './HeaderBar';

export class Header extends React.Component<HeaderBarProps> {
  static defaultProps = HeaderBar.defaultProps;

  render() {
    return (
      <div className="layout__header">
        <HeaderBar { ...this.props } />
      </div>
    );
  }
}


type ContainerProps = {
  children?: React.Element<*>
};

export class Container extends React.Component<ContainerProps> {
  render() {
    return (
      <div className="layout__container">
        { this.props.children }
      </div>
    );
  }
}

type LayoutProps = {
  children?: React.Node
};

export class Layout extends React.Component<LayoutProps> {
  render() {
    return (
      <div className="layout">
        { this.props.children }
      </div>
    );
  }
}
