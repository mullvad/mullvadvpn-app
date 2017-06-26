import React, { Component } from 'react';
import PropTypes from 'prop-types';

/**
 * A component used to chip out arrow in the app header using CSS mask
 *
 * @export
 * @class WindowChrome
 * @extends {Component}
 */
export default class WindowChrome extends Component {
  static propTypes = {
    children: PropTypes.oneOfType([
      PropTypes.arrayOf(PropTypes.element),
      PropTypes.element,
    ])
  };

  render() {
    const chromeClass = ['window-chrome', 'window-chrome--' + process.platform];
    return (
      <div className={ chromeClass.join(' ') }>
        { this.props.children }
      </div>
    );
  }
}