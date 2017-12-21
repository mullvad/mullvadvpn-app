// @flow
import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';

import Switch from './Switch';

import type { SettingsReduxState } from '../redux/settings/reducers';

export type onChangeLanSharingProps = {
  settings: SettingsReduxState;
  onChangeLanSharing: (boolean) => void;
  onClose: () => void;
};

export default class Preferences extends Component {
  props: onChangeLanSharingProps;

  render(): React.Element<*> {
    return (
      <Layout>
        <Header hidden={ true } style={ 'defaultDark' } />
        <Container>
          <div className="preferences">
            <div className="preferences__close" onClick={ this.props.onClose }>
              <img className="preferences__close-icon" src="./assets/images/icon-back.svg" />
              <span className="preferences__close-title">Settings</span>
            </div>
            <div className="preferences__container">

              <div className="preferences__header">
                <h2 className="preferences__title">Preferences</h2>
              </div>

              <div className="preferences__content">
                <div className="preferences__cell">
                  <div className="preferences__cell-label">Local network sharing</div>
                  <div className="preferences__cell-accessory">
                    <Switch isOn={ this.props.settings.allowLan } onChange={ this.props.onChangeLanSharing } />
                  </div>
                </div>
                <div className="preferences__cell-footer">
                  { 'Allows access to other devices on the same network for sharing, printing etc.' }
                </div>
              </div>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
