import React, { Component, PropTypes } from 'react';
import { TrayMenu, TrayItem } from '../lib/components/TrayMenu';
import { shell } from 'electron';

import { LoginState } from '../constants';

export default class Tray extends Component {

  static propTypes = {
    handle: PropTypes.object.isRequired,
    backend: PropTypes.object.isRequired
  }

  logout() {
    this.props.logout(this.props.backend);
  }

  openPrivacyPolicy() {
    shell.openExternal('https://mullvad.net/#privacy');
  }

  openHomepage() {
    shell.openExternal('https://mullvad.net');
  }
  
  render() {
    const loggedIn = this.props.user.status === LoginState.ok;

    return (
      <TrayMenu tray={ this.props.handle }>
        <TrayItem label="Log out" click={ ::this.logout } visible={ loggedIn } />
        <TrayItem type="separator" visible={ loggedIn } />
        <TrayItem label="Privacy Policy" click={ ::this.openPrivacyPolicy } />
        <TrayItem label="Visit homepage" click={ ::this.openHomepage } />
      </TrayMenu>
    );
  }

}