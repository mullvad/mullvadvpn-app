import React, { Component } from 'react';

export default class HeaderBar extends Component {
  render() {
    return (
      <div className="headerbar">
        <img className="headerbar__logo" src="./assets/images/logo-icon.svg" />
        <h2 className="headerbar__title">MULLVAD VPN</h2>
      </div>
    );
  }
}
