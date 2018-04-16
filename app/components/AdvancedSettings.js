// @flow

import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { Button } from './styled';
import { Layout, Container } from './Layout';
import CustomScrollbars from './CustomScrollbars';
import styles from './AdvancedSettingsStyles';
import Img from './Img';

export class AdvancedSettings extends Component {

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

      <View style={ styles.advanced_settings__cell_spacer }></View>

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

class Selector extends Component {

  props: {
    title: string,
    values: Array<*>,
    value: *,
    onSelect: (*) => void,
  }

  state = { hoveredButtonIndex: -1 };

  handleButtonHover = (value) => {
    this.setState({ hoveredButtonIndex: value });
  }

  render() {
    return <View>
      <View style={ styles.advanced_settings__section_title }>
        { this.props.title }
      </View>

      { this.props.values.map(value => this._renderCell(value)) }
    </View>;
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
    return <Button style={[ styles.advanced_settings__cell, value === this.state.hoveredButtonIndex ? styles.advanced_settings__cell_selected_hover : null ]}
      onPress={ () => this.props.onSelect(value) }
      onHoverStart={() => this.handleButtonHover(value)}
      onHoverEnd={() => this.handleButtonHover(-1)}
      key={ value }>
      <Img style={ styles.advanced_settings__cell_icon }
        source='icon-tick'
        tintColor='currentColor'/>
      <Text style={ styles.advanced_settings__cell_label }>{ value }</Text>
    </Button>;
  }

  _renderUnselectedCell(value) {
    return <Button style={[ styles.advanced_settings__cell_dimmed, value === this.state.hoveredButtonIndex ? styles.advanced_settings__cell_hover : null ]}
      onPress={ () => this.props.onSelect(value) }
      onHoverStart={() => this.handleButtonHover(value)}
      onHoverEnd={() => this.handleButtonHover(-1)}
      key={ value }>
      <View style={ styles.advanced_settings__cell_icon }></View>
      <Text style={ styles.advanced_settings__cell_label }>{ value }</Text>
    </Button>;
  }
}

function BaseLayout(props) {
  return <Layout>
    <Container>
      <View style={ styles.advanced_settings }>
        <Button style={ styles.advanced_settings__close }
          onPress={ props.onClose }
          testName='closeButton'>
          <View style={ styles.advanced_settings__close_content }>
            <Img height={24} width={24} style={ styles.advanced_settings__close_icon } source="icon-back" />
            <Text style={ styles.advanced_settings__close_title }>Settings</Text>
          </View>
        </Button>
        <View style={ styles.advanced_settings__container }>
          <View style={ styles.advanced_settings__header }>
            <Text style={ styles.advanced_settings__title }>Advanced</Text>
          </View>
          <CustomScrollbars style={styles.advanced_settings__scrollview} autoHide={ true }>
            <View style={ styles.advanced_settings__content }>
              { props.children }
            </View>
          </CustomScrollbars>
        </View>
      </View>
    </Container>
  </Layout>;
}

