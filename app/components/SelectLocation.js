import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { servers } from '../config';
import CustomScrollbars from './CustomScrollbars';

export default class SelectLocation extends Component {

  static propTypes = {
    onClose: PropTypes.func.isRequired,
    onSelect: PropTypes.func.isRequired
  }

  onSelect(name) {
    this.props.onSelect(name);
  }

  isSelected(key) {
    return key === this.props.settings.preferredServer;
  }

  drawCell(key, name, icon, onClick) {
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
  
  onCellRef(key, element) {
    // save reference to selected cell 
    if(this.isSelected(key)) {
      this._selectedCell = element;
    }
  }
  
  componentDidMount() {
    // restore scroll to selected cell
    if(this._selectedCell) {
      this._selectedCell.scrollIntoViewIfNeeded(true);
    }
  }

  render() {
    return (
      <Layout>
        <Header hidden={ true } style={ Header.Style.defaultDark } />
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
                  
                  { this.drawCell('fastest', 'Fastest', './assets/images/icon-fastest.svg', this.onSelect.bind(this, 'fastest')) }
                  { this.drawCell('nearest', 'Nearest', './assets/images/icon-nearest.svg', this.onSelect.bind(this, 'nearest')) }

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
