import React, { Component, PropTypes } from 'react';
import { Scrollbars } from 'react-custom-scrollbars';

export default class CustomScrollbars extends Component {
  static propTypes: {
    children: PropTypes.element
  }

  render() {
    return (
      <Scrollbars
        { ...this.props }
        renderThumbVertical={ props => <div className="custom-scrollbars__thumb-vertical"/> }>
        { this.props.children }
      </Scrollbars>
    );
  }
}
