// @flow
import React, { Component } from 'react';

export default class WindowChrome extends Component {
  props: {
    children: Array<React.Element<*>> | React.Element<*>
  }

  render() {
    const chromeClass = ['window-chrome', 'window-chrome--' + process.platform];
    return (
      <div className={ chromeClass.join(' ') }>
        { this.props.children }
      </div>
    );
  }
}