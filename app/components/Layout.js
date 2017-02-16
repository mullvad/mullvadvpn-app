import React, { PropTypes, Component } from 'react';
import HeaderBar from './HeaderBar';

export class Header extends Component {

  static Style = HeaderBar.Style

  render() {
    return (
      <div className="layout__header">
        <HeaderBar {...this.props} />
      </div>
    );
  }
}

export class Container extends Component {
  static propTypes = {
    children: PropTypes.element.isRequired
  };

  render() {
    return (
      <div className="layout__container">
        {this.props.children}
      </div>
    );
  }
}

export class Layout extends Component {
  static propTypes = {
    children: PropTypes.arrayOf(PropTypes.node).isRequired
  };
  render() {
    return (
      <div className="layout">
        {this.props.children}
      </div>
    );
  }
}
