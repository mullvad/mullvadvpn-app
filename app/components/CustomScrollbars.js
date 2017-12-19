// @flow
import React, { Component } from 'react';
import { Scrollbars } from 'react-custom-scrollbars';

export default class CustomScrollbars extends Component {
  props: {
    children: ?React.Element<*>
  }

  render(): React.Element<*> {
    return (
      <Scrollbars
        { ...this.props }
        renderThumbVertical={ () => <div className="custom-scrollbars__thumb-vertical"/> }>
        { this.props.children }
      </Scrollbars>
    );
  }
}
