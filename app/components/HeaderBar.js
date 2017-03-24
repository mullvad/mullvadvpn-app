import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import Enum from '../lib/enum';

/**
 * Header bar component
 * 
 * @export
 * @class HeaderBar
 * @extends {React.Component}
 */
export default class HeaderBar extends Component {

  /**
   * Bar style
   * @type {Style}
   * @property {string} default     - default
   * @property {string} defaultDark - default dark blue
   * @property {string} error       - red
   * @property {string} success     - green
   * @static
   * 
   * @memberOf HeaderBar
   */
  static Style = new Enum('default', 'defaultDark', 'error', 'success');

  /**
   * Prop types
   * @static
   * 
   * @memberOf HeaderBar
   */
  static propTypes = {
    style: PropTypes.string,
    hidden: PropTypes.bool,
    showSettings: PropTypes.bool,
    onSettings: PropTypes.func
  };

  /**
   * @override
   */
  render() {
    const style = this.props.style;
    let containerClass = ['headerbar', 'headerbar--' + process.platform];

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
