// @flow
import React, { Component } from 'react';
import { If, Then } from 'react-if';

export type HeaderBarStyle = 'default' | 'defaultDark' | 'error' | 'success';
export type HeaderBarProps = {
  style: HeaderBarStyle;
  hidden: boolean;
  showSettings: boolean;
  onSettings: ?(() => void);
};

export default class HeaderBar extends Component {
  props: HeaderBarProps;
  static defaultProps: $Shape<HeaderBarProps> = {
    hidden: false,
    showSettings: false
  };

  render(): React.Element<*> {
    let containerClass = [
      'headerbar',
      'headerbar--' + process.platform,
      'headerbar--style-' + this.props.style
    ];

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
