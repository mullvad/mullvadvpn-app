import React, { PropTypes, Component } from 'react';
import HeaderBar from './HeaderBar';

/**
 * Layout header
 * 
 * @export
 * @class Header
 * @extends {Component}
 */
export class Header extends Component {

  static Style = HeaderBar.Style

  render() {
    return (
      <div className="layout__header">
        <HeaderBar { ...this.props } />
      </div>
    );
  }
}

/**
 * Content container
 * 
 * @export
 * @class Container
 * @extends {Component}
 */
export class Container extends Component {
  static propTypes = {
    children: PropTypes.element.isRequired
  };

  render() {
    return (
      <div className="layout__container">
        { this.props.children }
      </div>
    );
  }
}

/**
 * Layout container
 * 
 * @export
 * @class Layout
 * @extends {Component}
 */
export class Layout extends Component {
  static propTypes = {
    children: PropTypes.oneOfType([
      PropTypes.arrayOf(PropTypes.element),
      PropTypes.element,
    ])
  };
  
  render() {
    return (
      <div className="layout">
        { this.props.children }
      </div>
    );
  }
}
