// @flow

import React from 'react';
import { Layout, Container, Header } from './Layout';
import CustomScrollbars from './CustomScrollbars';

import TickSVG from '../assets/images/icon-tick.svg';

export class AdvancedSettings extends React.Component {

  props: {
    protocol: string,
    port: string | number,
    onUpdate: (protocol: string, port: string | number) => void,
    onClose: () => void,
  };

  render() {
    let portSelector = null;
    let protocol = this.props.protocol.toUpperCase();

    if (protocol === 'AUTOMATIC') {
      protocol = 'Automatic';
    } else {
      portSelector = this._createPortSelector();
    }

    return <BaseLayout onClose={ this.props.onClose }>

      <Selector
        title={ 'Network protocols' }
        values={ ['Automatic', 'UDP', 'TCP'] }
        value={ protocol }
        onSelect={ protocol => {
          this.props.onUpdate(protocol, 'Automatic');
        }}/>

      <div className="settings__cell-spacer"></div>

      { portSelector }

    </BaseLayout>;
  }

  _createPortSelector() {
    const protocol = this.props.protocol.toUpperCase();
    const ports = protocol === 'TCP'
      ? ['Automatic', 80, 443]
      : ['Automatic', 1194, 1195, 1196, 1197, 1300, 1301, 1302];

    return <Selector
      title={ protocol + ' port' }
      values={ ports }
      value={ this.props.port }
      onSelect={ port => {
        this.props.onUpdate(protocol, port);
      }} />;
  }
}


class Selector extends React.Component {

  props: {
    title: string,
    values: Array<*>,
    value: *,
    onSelect: (*) => void,
  }

  render() {
    return <div>
      <div className="advanced-settings__section-title">
        { this.props.title }
      </div>

      { this.props.values.map(value => this._renderCell(value)) }
    </div>;
  }

  _renderCell(value) {
    const selected = value === this.props.value;
    if (selected) {
      return this._renderSelectedCell(value);
    } else {
      return this._renderUnselectedCell(value);
    }
  }

  _renderSelectedCell(value) {
    return <div
      key={ value }
      className="advanced-settings__cell advanced-settings__cell--selected"
      onClick={ () => this.props.onSelect(value) } >
      <div className="advanced-settings__cell-icon"><TickSVG /></div>
      <div className="advanced-settings__cell-label">{ value }</div>
    </div>;
  }

  _renderUnselectedCell(value) {
    return <div
      key={ value }
      className="advanced-settings__cell advanced-settings__cell--dimmed"
      onClick={ () => this.props.onSelect(value) }>
      <div className="advanced-settings__cell-icon"></div>
      <div className="advanced-settings__cell-label">{ value }</div>
    </div>;
  }
}

function BaseLayout(props) {
  return <Layout>
    <Header hidden={ true } style={ 'defaultDark' } />
    <Container>
      <div className="settings">
        <div className="support__close" onClick={ props.onClose }>
          <img className="support__close-icon" src="./assets/images/icon-back.svg" />
          <span className="support__close-title">Settings</span>
        </div>
        <div className="settings__container">
          <div className="settings__header">
            <h2 className="settings__title">Advanced</h2>
          </div>
          <CustomScrollbars autoHide={ true }>
            <div className="settings__content">
              <div className="settings__main">
                { props.children }
              </div>
            </div>
          </CustomScrollbars>
        </div>
      </div>
    </Container>
  </Layout>;
}

