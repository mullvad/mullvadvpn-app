// @flow

import * as React from 'react';
import { Button, Component, Text, View } from 'reactxp';
import * as Cell from './Cell';
import { Layout, Container } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  BackBarItem,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import Switch from './Switch';
import styles from './AdvancedSettingsStyles';
import Img from './Img';

type Props = {
  enableIpv6: boolean,
  protocol: string,
  port: string | number,
  setEnableIpv6: (boolean) => void,
  onUpdate: (protocol: string, port: string | number) => void,
  onClose: () => void,
};

export class AdvancedSettings extends Component<Props> {
  render() {
    let portSelector = null;
    let protocol = this.props.protocol.toUpperCase();

    if (protocol === 'AUTOMATIC') {
      protocol = 'Automatic';
    } else {
      portSelector = this._createPortSelector();
    }

    return (
      <Layout>
        <Container>
          <View style={styles.advanced_settings}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose} title={'Settings'} />
                <TitleBarItem>Advanced</TitleBarItem>
              </NavigationBar>

              <View style={styles.advanced_settings__container}>
                <NavigationScrollbars style={styles.advanced_settings__scrollview}>
                  <SettingsHeader>
                    <HeaderTitle>Advanced</HeaderTitle>
                  </SettingsHeader>

                  <Cell.Container>
                    <Cell.Label>Enable IPv6</Cell.Label>
                    <Switch isOn={this.props.enableIpv6} onChange={this.props.setEnableIpv6} />
                  </Cell.Container>
                  <Cell.Footer>Enable IPv6 communication through the tunnel.</Cell.Footer>

                  <View style={styles.advanced_settings__content}>
                    <Selector
                      title={'Network protocols'}
                      values={['Automatic', 'UDP', 'TCP']}
                      value={protocol}
                      onSelect={(protocol) => {
                        this.props.onUpdate(protocol, 'Automatic');
                      }}
                    />

                    <View style={styles.advanced_settings__cell_spacer} />

                    {portSelector}
                  </View>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  _createPortSelector() {
    const protocol = this.props.protocol.toUpperCase();
    const ports =
      protocol === 'TCP'
        ? ['Automatic', 80, 443]
        : ['Automatic', 1194, 1195, 1196, 1197, 1300, 1301, 1302];

    return (
      <Selector
        title={protocol + ' port'}
        values={ports}
        value={this.props.port}
        onSelect={(port) => {
          this.props.onUpdate(protocol, port);
        }}
      />
    );
  }
}

type SelectorProps<T> = {
  title: string,
  values: Array<T>,
  value: T,
  onSelect: (T) => void,
};

type SelectorState = {
  hoveredButtonIndex: number,
};

class Selector extends Component<SelectorProps<*>, SelectorState> {
  state = { hoveredButtonIndex: -1 };

  handleButtonHover = (value) => {
    this.setState({ hoveredButtonIndex: value });
  };

  render() {
    return (
      <View>
        <View style={styles.advanced_settings__section_title}>{this.props.title}</View>

        {this.props.values.map((value) => this._renderCell(value))}
      </View>
    );
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
    return (
      <Button
        style={[
          styles.advanced_settings__cell,
          value === this.state.hoveredButtonIndex
            ? styles.advanced_settings__cell_selected_hover
            : null,
        ]}
        onPress={() => this.props.onSelect(value)}
        onHoverStart={() => this.handleButtonHover(value)}
        onHoverEnd={() => this.handleButtonHover(-1)}
        key={value}>
        <Img
          style={styles.advanced_settings__cell_icon}
          source="icon-tick"
          tintColor="currentColor"
        />
        <Text style={styles.advanced_settings__cell_label}>{value}</Text>
      </Button>
    );
  }

  _renderUnselectedCell(value) {
    return (
      <Button
        style={[
          styles.advanced_settings__cell_dimmed,
          value === this.state.hoveredButtonIndex ? styles.advanced_settings__cell_hover : null,
        ]}
        onPress={() => this.props.onSelect(value)}
        onHoverStart={() => this.handleButtonHover(value)}
        onHoverEnd={() => this.handleButtonHover(-1)}
        key={value}>
        <View style={styles.advanced_settings__cell_icon} />
        <Text style={styles.advanced_settings__cell_label}>{value}</Text>
      </Button>
    );
  }
}
