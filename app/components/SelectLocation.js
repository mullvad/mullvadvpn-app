// @flow
import React, { Component } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { servers } from '../config';
import CustomScrollbars from './CustomScrollbars';

import type { SettingsReduxState } from '../redux/settings/reducers';

export type SelectLocationProps = {
  settings: SettingsReduxState,
  onClose: () => void;
  onSelect: (server: string) => void;
};

export default class SelectLocation extends Component {
  props: SelectLocationProps;
  _selectedCell: ?HTMLElement;

  onSelect(name: string) {
    if (!this.isSelected(name)) {
      this.props.onSelect(name);
    }
  }

  isSelected(server: string) {
    const { host } = this.props.settings.relayConstraints;
    return server === host;
  }

  drawCell(key: string, name: string, icon: ?string, onClick: (e: Event) => void): React.Element<*> {
    const classes = ['select-location__cell'];
    const selected = this.isSelected(key);

    if(selected) {
      classes.push('select-location__cell--selected');
    }

    const cellClass = classes.join(' ');

    return (
      <div key={ key } className={ cellClass } onClick={ onClick } ref={ (e) => this.onCellRef(key, e) }>

        <If condition={ !!icon }>
          <Then>
            <img className="select-location__cell-icon" src={ icon } />
          </Then>
        </If>

        <div className="select-location__cell-label">{ name }</div>

        <If condition={ selected } >
          <Then>
            <img className="select-location__cell-accessory" src="./assets/images/icon-tick.svg" />
          </Then>
        </If>

      </div>
    );
  }

  onCellRef(key: string, element: HTMLElement) {
    // save reference to selected cell
    if(this.isSelected(key)) {
      this._selectedCell = element;
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

                  { Object.keys(servers).map((key) => this.drawCell(key, servers[key].name, null, this.onSelect.bind(this, key))) }

                </div>
              </CustomScrollbars>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
