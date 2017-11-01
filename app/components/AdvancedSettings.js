// @flow

import React from 'react';
import { Layout, Container, Header } from './Layout';
import CustomScrollbars from './CustomScrollbars';

type Props = {
  onClose: () => void,
  protocol: string,
  port: string|number,
  updateConstraints: (string, string|number) => void,
};
export function AdvancedSettings(props: Props) {

  let portSelector = null;
  let protocol = props.protocol.toUpperCase();

  if (protocol === 'AUTOMATIC') {
    protocol = 'Automatic';
  } else {
    portSelector = createPortSelector(props);
  }

  return <BaseLayout onClose={ props.onClose }>

    <Selector
      title={ 'Network protocols' }
      values={ ['Automatic', 'UDP', 'TCP'] }
      value={ protocol }
      onSelect={ protocol => {
        // $FlowFixMe
        props.updateConstraints(protocol, 'Automatic');
      }}/>

    <div className="settings__cell-spacer"></div>

    { portSelector }

  </BaseLayout>;

}

function createPortSelector(props) {
  const protocol = props.protocol.toUpperCase();
  const ports = protocol === 'TCP'
    ? ['Automatic', 80, 443]
    : ['Automatic', 1194, 1195, 1196, 1197, 1300, 1301, 1302];

  return <Selector
    title={ protocol + ' port' }
    values={ ports }
    value={ props.port }
    onSelect={ port => {
      props.updateConstraints(protocol, port);
    }} />;
}

function Selector(props) {
  return <div>
    <Cell
      label={ props.title }
    />

    { props.values.map(value => renderCell(value)) }
  </div>;

  function renderCell(value) {
    const selected = value === props.value;

    let className = 'settings__sub-cell';
    let tick = null;
    if (selected) {
      className = 'settings__cell--selected';
      tick = <img src='./assets/images/icon-tick.svg' />;
    }
    const label = <div className={ 'settings__sub-cell--label' }>
      { tick }
      { value }
    </div>;

    const onCellClick = () => props.onSelect(value);

    return <Cell
      key={ value }
      label={ label }
      className={ className }
      onClick={ onCellClick } />;
  }
}

function BaseLayout(props) {
  return <Layout>
    <Header hidden={ true } style={ 'defaultDark' } />
    <Container>
      <div className="settings">
        <button className="settings__close" onClick={ props.onClose } />
        <div className="settings__container">
          <div className="settings__header">
            <h2 className="settings__title">Advanced Settings</h2>
          </div>
          <CustomScrollbars autoHide={ true }>
            <div className="settings__content">
              <div className="settings__main">
                <div className="settings__advanced">
                  { props.children }
                </div>
              </div>
            </div>
          </CustomScrollbars>
        </div>
      </div>
    </Container>
  </Layout>;
}

function Cell(props) {

  const className = props.className || '';
  return <div
    className={ className + ' settings__cell' }
    onClick={ props.onClick || null } >
    <div className="settings__cell-label">{ props.label }</div>
    <div className="settings__cell-value">
      { props.value || null }
    </div>
  </div>;
}
