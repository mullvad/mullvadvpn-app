// @flow
import React, { Component } from 'react';

export default class PlatformWindow extends Component {
  props: {
    children: Array<React.Element<*>> | React.Element<*>
  }
  render(): React.Element<*> {
    const chromeClass = ['window-chrome', 'window-chrome--' + process.platform];
    return (
      <div className={ chromeClass.join(' ') }>
        { this.props.children }
      </div>
    );
  }
}