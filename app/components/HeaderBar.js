import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import Enum from '../lib/enum';

export default class HeaderBar extends Component {

  /** Bar style */
  static Style = Enum('default', 'defaultDark', 'error', 'success');

  static propTypes = {
    style: PropTypes.string,
    hidden: PropTypes.bool,
    showSettings: PropTypes.bool,
    onSettings: PropTypes.func
  };

  render() {
    const style = this.props.style;
    let containerClass = ['headerbar'];

    if(HeaderBar.Style.isValid(style)) {
      containerClass.push(`headerbar--style-${style}`);
    }

    if(this.props.hidden) {
      containerClass.push('headerbar--hidden');
    }

    return (
      <div className={ containerClass.join(' ') }>
        <If condition={ !this.props.hidden }>
          <Then>
            <div className="headerbar__container">
              <img className="headerbar__logo" src="./assets/images/logo-icon.svg" />
              <h2 className="headerbar__title">MULLVAD VPN</h2>
              <If condition={ !!this.props.showSettings }>
                <Then>
                  <button className="headerbar__settings" onClick={ this.props.onSettings } />
                </Then>
              </If>
            </div>
          </Then>
        </If>
      </div>
    );
  }
}
