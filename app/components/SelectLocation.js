// @flow
import * as React from 'react';
import { Layout, Container, Header } from './Layout';
import CustomScrollbars from './CustomScrollbars';

import Accordion from './Accordion';
import ChevronDownSVG from '../assets/images/icon-chevron-down.svg';
import ChevronUpSVG from '../assets/images/icon-chevron-up.svg';
import TickSVG from '../assets/images/icon-tick.svg';

import type { SettingsReduxState, RelayLocationRedux, RelayLocationCityRedux } from '../redux/settings/reducers';
import type { RelayLocation } from '../lib/ipc-facade';

export type SelectLocationProps = {
  settings: SettingsReduxState,
  onClose: () => void;
  onSelect: (location: RelayLocation) => void;
};

type State = {
  expanded: Array<string>
};

export default class SelectLocation extends React.Component<SelectLocationProps, State> {
  _selectedCell: ?HTMLElement;

  state = {
    expanded: [],
  };

  constructor(props: SelectLocationProps, context?: any) {
    super(props, context);

    // set initially expanded country based on relaySettings
    const relaySettings = this.props.settings.relaySettings;
    if(relaySettings.normal) {
      const { location } = relaySettings.normal;
      if(location === 'any') {
        // no-op
      } else if(location.country) {
        this.state.expanded.push(location.country);
      } else if(location.city) {
        this.state.expanded.push(location.city[0]);
      }
    }
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

  render() {
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

                  { this.props.settings.relayLocations.map((relayCountry) => {
                    return this._renderCountry(relayCountry);
                  }) }

                </div>
              </CustomScrollbars>
            </div>
          </div>
        </Container>
      </Layout>
    );
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

  _toggleCollapse = (countryCode: string) => {
    this.setState((state) => {
      const expanded = state.expanded.slice();
      const index = expanded.indexOf(countryCode);
      if(index === -1) {
        expanded.push(countryCode);
      } else {
        expanded.splice(index, 1);
      }
      return { expanded };
    });
  }

  _relayStatusIndicator(active: boolean) {
    const statusClass = active ? 'select-location-relay-status--active' : 'select-location-relay-status--inactive';

    return (<div className={ 'select-location-relay-status ' + statusClass }></div>);
  }

  _renderCountry(relayCountry: RelayLocationRedux) {
    const isSelected = this._isSelected({ country: relayCountry.code });

    // either expanded by user or when the city selected within the country
    const isExpanded = this.state.expanded.includes(relayCountry.code);

    const handleSelect = (relayCountry.hasActiveRelays && !isSelected) ? () => {
      this.props.onSelect({ country: relayCountry.code });
    } : undefined;

    const handleCollapse = (e) => {
      this._toggleCollapse(relayCountry.code);
      e.stopPropagation();
    };

    const countryClass = 'select-location__cell' +
      (isSelected ? ' select-location__cell--selected' : '') +
      (relayCountry.hasActiveRelays ? ' select-location__cell--selectable' : '');

    const onRef = isSelected ? (element) => {
      this._selectedCell = element;
    } : undefined;

    return (
      <div key={ relayCountry.code } className="select-location__country">
        <div className={ countryClass }
          onClick={ handleSelect }
          ref={ onRef }>
          <div className="select-location__cell-content">

            <div className="select-location__cell-icon">
              { isSelected ?
                <TickSVG /> :
                this._relayStatusIndicator(relayCountry.hasActiveRelays) }
            </div>

            <div className={ 'select-location__cell-label' +
              (relayCountry.hasActiveRelays ? '' : ' select-location__cell-label--inactive') }>
              { relayCountry.name }
            </div>
          </div>

          { relayCountry.cities.length > 1 && <button type="button" className="select-location__collapse-button" onClick={ handleCollapse }>
            { isExpanded ?
              <ChevronUpSVG className="select-location__collapse-icon" /> :
              <ChevronDownSVG className="select-location__collapse-icon" /> }
          </button> }

        </div>

        { relayCountry.cities.length > 1 &&
          (<Accordion className="select-location__cities" height={ isExpanded ? 'auto' : 0 }>
            { relayCountry.cities.map((relayCity) => this._renderCity(relayCountry.code, relayCity)) }
          </Accordion>)
        }
      </div>
    );
  }

  _renderCity(countryCode: string, relayCity: RelayLocationCityRedux) {
    const relayLocation: RelayLocation = { city: [countryCode, relayCity.code] };

    const isSelected = this._isSelected(relayLocation);

    const cityClass = 'select-location__sub-cell' +
      (isSelected ? ' select-location__sub-cell--selected' : '') +
      (relayCity.hasActiveRelays ? ' select-location__sub-cell--selectable' : '');

    const handleSelect = (relayCity.hasActiveRelays && !isSelected) ? () => {
      this.props.onSelect(relayLocation);
    } : undefined;

    const onRef = isSelected ? (element) => {
      this._selectedCell = element;
    } : undefined;

    return (
      <div key={ `${countryCode}_${relayCity.code}` }
        className={ cityClass }
        onClick={ handleSelect }
        ref={ onRef }>

        <div className="select-location__cell-icon">
          { isSelected ?
            <TickSVG /> :
            this._relayStatusIndicator(relayCity.hasActiveRelays) }
        </div>

        <div className={ 'select-location__cell-label' +
          (relayCity.hasActiveRelays ? '' : ' select-location__cell-label--inactive') }>
          { relayCity.name }
        </div>
      </div>
    );
  }

}
