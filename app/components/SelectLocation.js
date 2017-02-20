import React, { Component, PropTypes } from 'react';
import { Layout, Container, Header } from './Layout';
import { servers } from '../constants';

export default class SelectLocation extends Component {

  static propTypes = {
    updateSettings: PropTypes.func.isRequired
  }

  onClose() {
    this.props.router.push('/connect');
  }

  handleSelection(name) {
    console.log('Selected: ', name);
  }

  handleFastest() {
    console.log('Selected: FASTEST');
  }

  handleNearest() {
    console.log('Selected: NEAREST');
  }

  render() {
    return (
      <Layout>
        <Header hidden={ true } style={ Header.Style.defaultDark } />
        <Container>
          <div className="select-location">
            <button className="select-location__close" onClick={ ::this.onClose } />
            <div className="select-location__container">
              <div className="select-location__header">
                <h2 className="select-location__title">Select location</h2>
                <div className="select-location__subtitle">
                  While connected, your real location is masked with a private and secure location in the selected region
                </div>
              </div>
              
              <div className="select-location__list">
                <div>
                  <div className="select-location__cell" onClick={ ::this.handleFastest }>
                    <img className="select-location__cell-icon" src="./assets/images/icon-fastest.svg" />
                    <div className="select-location__cell-label">Fastest</div>
                  </div>
                  <div className="select-location__cell" onClick={ ::this.handleNearest }>
                    <img className="select-location__cell-icon" src="./assets/images/icon-nearest.svg" />
                    <div className="select-location__cell-label">Nearest</div>
                  </div>
                  <div className="select-location__separator"></div>
                  
                  {
                    servers.map((name) => (
                    <div className="select-location__cell" key={ name } onClick={ this.handleSelection.bind(this, name) }>
                      <div className="select-location__cell-label">{ name }</div>
                    </div>))
                  }

                </div>
              </div>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
