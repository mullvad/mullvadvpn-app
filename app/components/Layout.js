import React, { PropTypes, Component } from 'react';
import HeaderBar from './HeaderBar';

export default class Layout extends Component {
  static propTypes = {
    children: PropTypes.element.isRequired
  };
  render() {
    return (
      <div className="layout">
        <div className="layout__header">
          <HeaderBar />
        </div>
        <div className="layout__container">
          {this.props.children}
        </div>
      </div>
    );
  }
}
