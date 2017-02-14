import React, { Component, PropTypes } from 'react';
import Enum from '../lib/enum';

export default class HeaderBar extends Component {

  /** Bar style */
  static Style = Enum('default', 'error');

  static propTypes = {
    style: PropTypes.string
  };

  render() {
    const style = this.props.style;
    let containerClass = ['headerbar'];

    if(HeaderBar.Style.isValid(style)) {
      containerClass.push(`header--style-${style}`);
    }

    return (
      <div className={ containerClass.join(' ') }>
        <img className="headerbar__logo" src="./assets/images/logo-icon.svg" />
        <h2 className="headerbar__title">MULLVAD VPN</h2>
      </div>
    );
  }
}
