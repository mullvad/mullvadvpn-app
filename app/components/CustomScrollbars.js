import React, { Component, PropTypes } from 'react';
import { Scrollbars } from 'react-custom-scrollbars';

/**
 * Custom scrollbars component
 *
 * @export
 * @class CustomScrollbars
 * @extends {React.Component}
 */
export default class CustomScrollbars extends Component {
  /**
   * PropTypes
   * @static
   * @memberOf CustomScrollbars
   */
  static propTypes = {
    children: PropTypes.element
  }

  /**
   * @override
   */
  render() {
    return (
      <Scrollbars
        { ...this.props }
        renderThumbVertical={ () => <div className="custom-scrollbars__thumb-vertical"/> }>
        { this.props.children }
      </Scrollbars>
    );
  }
}
