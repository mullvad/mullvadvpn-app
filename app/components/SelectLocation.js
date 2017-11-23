// @flow
import React, { Component } from 'react';
import { Layout, Container, Header } from './Layout';
import CustomScrollbars from './CustomScrollbars';
import { servers } from '../config';

import type { SettingsReduxState } from '../redux/settings/reducers';
import type { RelayLocation } from '../lib/ipc-facade';

export type SelectLocationProps = {
  settings: SettingsReduxState,
  onClose: () => void;
  onSelect: (location: RelayLocation) => void;
};

export default class SelectLocation extends Component {
  props: SelectLocationProps;
  _selectedCell: ?HTMLElement;

  _onSelect(location: RelayLocation) {
    if (!this._isSelected(location)) {
      this.props.onSelect(location);
    }
  }

  _isSelected(selectedLocation: RelayLocation) {
    const { relaySettings } = this.props.settings;
    if(relaySettings.normal) {
      const otherLocation = relaySettings.normal.location;

      if(selectedLocation.country && otherLocation.country &&
        selectedLocation.country === otherLocation.country) {
        return true;
      }

      if(Array.isArray(selectedLocation.city) && Array.isArray(otherLocation.city)) {
        const selectedCity = selectedLocation.city;
        const otherCity = otherLocation.city;

        return selectedCity.length === otherCity.length &&
              selectedCity.every((v, i) => v === otherCity[i]);
      }
    }
    return false;
  }

  drawCell(key: string, name: string, selected: bool, icon: ?string, onClick: (e: Event) => void): React.Element<*> {
    const classes = ['select-location__cell'];
    if(selected) {
      classes.push('select-location__cell--selected');
    }
    const cellClass = classes.join(' ');
    const onRef = selected ? (element) => {
      this._selectedCell = element;
    } : undefined;

    return (
      <div key={ key } className={ cellClass } onClick={ onClick } ref={ onRef }>

        { icon && <img className="select-location__cell-icon" src={ icon } />}

        <div className="select-location__cell-label">{ name }</div>

        { selected && <img className="select-location__cell-accessory" src="./assets/images/icon-tick.svg" /> }

      </div>
    );
  }

  componentDidMount() {
    // restore scroll to selected cell
    const cell = this._selectedCell;
    if(cell) {
      // this is non-standard webkit method but it works great!
      if(typeof(cell.scrollIntoViewIfNeeded) !== 'function') {
        console.warn('HTMLElement.scrollIntoViewIfNeeded() is not available anymore! Please replace it with viable alternative.');
        return;
      }
      cell.scrollIntoViewIfNeeded(true);
    }
  }

  render(): React.Element<*> {
    return (
      <Layout>
        <Header hidden={ true } style={ 'defaultDark' } />
        <Container>
          <div className="select-location">
            <button className="select-location__close" onClick={ this.props.onClose } />
            <div className="select-location__container">
              <div className="select-location__header">
                <h2 className="select-location__title">Select location</h2>
              </div>

              <CustomScrollbars autoHide={ true }>
                <div>
                  <div className="select-location__subtitle">
                    While connected, your real location is masked with a private and secure location in the selected region
                  </div>

                  <div className="select-location__separator"></div>

                  { Object.keys(servers).map((key) => {
                    const { name, country_code, city_code } = servers[key];
                    const location = {
                      city: [ country_code, city_code ]
                    };
                    const selected = this._isSelected(location);
                    const clickHandler = () => this._onSelect(location);
                    return this.drawCell(key, name, selected, null, clickHandler);
                  }) }

                </div>
              </CustomScrollbars>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
