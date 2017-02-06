import React, { Component, PropTypes } from 'react';
import { TrayMenu, TrayItem } from '../lib/components/TrayMenu';

export default class Tray extends Component {

  static propTypes = {
    handle: PropTypes.object.isRequired,
  }

  logout() {
    this.props.login({ username: '', loggedIn: false });
    this.props.history.push('/');
  }
  
  render() {
    const loggedIn = this.props.user && this.props.user.loggedIn;

    return (
      <TrayMenu tray={this.props.handle}>
        {loggedIn && <TrayItem label="Log out" click={::this.logout} />}
        {loggedIn && <TrayItem type="separator" />}
        <TrayItem label="Privacy Policy" />
        <TrayItem label="Visit homepage" />
      </TrayMenu>
    );
  }

}