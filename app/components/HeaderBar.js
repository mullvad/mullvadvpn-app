import React, { Component } from 'react';

export default class HeaderBar extends Component {
  render() {
    return (
      <div className="headerbar">
        <img className="headerbar__logo" src="data:image/svg+xml;charset=utf-8,%3Csvg xmlns%3D'http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg' viewBox%3D'0 0 50 50'%2F%3E" />
        <h2 className="headerbar__title">MULLVAD VPN</h2>
      </div>
    );
  }
}
