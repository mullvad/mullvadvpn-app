// @flow
import React, { Component } from 'react';
import HeaderBar from './HeaderBar';

import type { HeaderBarProps } from './HeaderBar';

export class Header extends Component {
  props: HeaderBarProps;
  static defaultProps = HeaderBar.defaultProps;

  render(): React.Element<*> {
    return (
      <div className="layout__header">
        <HeaderBar { ...this.props } />
      </div>
    );
  }
}

export class Container extends Component {
  props: {
    children: React.Element<*>
  }

  render(): React.Element<*> {
    return (
      <div className="layout__container">
        { this.props.children }
      </div>
    );
  }
}

export class Layout extends Component {
  props: {
    children: Array<React.Element<*>> | React.Element<*>
  }

  render(): React.Element<*> {
    return (
      <div className="layout">
        { this.props.children }
      </div>
    );
  }
}
