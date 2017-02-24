import React, { Component, PropTypes } from 'react';
import { If, Then } from 'react-if';
import { Layout, Container, Header } from './Layout';
import { servers } from '../constants';
import CustomScrollbars from './CustomScrollbars';

export default class SelectLocation extends Component {

  static propTypes = {
    onChangeLocation: PropTypes.func.isRequired
  }

  onClose() {
    this.props.router.push('/connect');
  }

  handleSelection(name) {
    this.props.onChangeLocation(name);
    this.props.router.push('/connect');
  }

  handleFastest() {
    this.props.onChangeLocation('fastest');
    this.props.router.push('/connect');
  }

  handleNearest() {
    this.props.onChangeLocation('nearest');
    this.props.router.push('/connect');
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
            <button className="select-location__close" onClick={ ::this.onClose } />
            <div className="select-location__container">
              <div className="select-location__header">
                <h2 className="select-location__title">Select location</h2>
                <div className="select-location__subtitle">
                  While connected, your real location is masked with a private and secure location in the selected region
                </div>
              </div>
              
              <CustomScrollbars autoHide={ true }>
                <div>
                  { this.drawCell('fastest', 'Fastest', './assets/images/icon-fastest.svg', ::this.handleFastest) }
                  { this.drawCell('nearest', 'Nearest', './assets/images/icon-nearest.svg', ::this.handleNearest) }

                  <div className="select-location__separator"></div>
                  
                  { Object.keys(servers).map((key) => this.drawCell(key, servers[key].name, null, this.handleSelection.bind(this, key))) }

                </div>
              </CustomScrollbars>
            </div>
          </div>
        </Container>
      </Layout>
    );
  }
}
