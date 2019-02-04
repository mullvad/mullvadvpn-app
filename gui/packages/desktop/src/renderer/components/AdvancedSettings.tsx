/* tslint:disable:jsx-no-lambda */
// TODO: Refactor this file to fix the jsx-no-lambda warnings

import { HeaderTitle, ImageView, SettingsHeader } from '@mullvad/components';
import * as React from 'react';
import { Button, Component, Text, View } from 'reactxp';
import { colors } from '../../config.json';
import styles from './AdvancedSettingsStyles';
import * as Cell from './Cell';
import { Container, Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import Switch from './Switch';

const MIN_MSSFIX_VALUE = 1000;
const MAX_MSSFIX_VALUE = 1450;

interface IProps {
  enableIpv6: boolean;
  blockWhenDisconnected: boolean;
  protocol: string;
  mssfix?: number;
  port: string | number;
  setEnableIpv6: (value: boolean) => void;
  setBlockWhenDisconnected: (value: boolean) => void;
  setOpenVpnMssfix: (value: number | undefined) => void;
  onUpdate: (protocol: string, port: string | number) => void;
  onClose: () => void;
}

interface IState {
  persistedMssfix?: number;
  editedMssfix?: number;
  focusOnMssfix: boolean;
}

export class AdvancedSettings extends Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    this.state = {
      persistedMssfix: props.mssfix,
      editedMssfix: props.mssfix,
      focusOnMssfix: false,
    };
  }

  public componentDidUpdate(_oldProps: IProps, _oldState: IState) {
    if (this.props.mssfix !== this.state.persistedMssfix) {
      this.setState((state, props) => ({
        ...state,
        persistedMssfix: props.mssfix,
        editedMssfix: state.focusOnMssfix ? state.editedMssfix : props.mssfix,
      }));
    }
  }

  public render() {
    let portSelector = null;
    let protocol = this.props.protocol.toUpperCase();

    if (protocol === 'AUTOMATIC') {
      protocol = 'Automatic';
    } else {
      portSelector = this.createPortSelector();
    }

    const mssfixStyle = this.mssfixIsValid()
      ? styles.advanced_settings__mssfix_valid_value
      : styles.advanced_settings__mssfix_invalid_value;

    const mssfixValue = this.state.editedMssfix;

    return (
      <Layout>
        <Container>
          <View style={styles.advanced_settings}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose}>Settings</BackBarItem>
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

                  <Cell.Container>
                    <Cell.Label textStyle={styles.advanced_settings__block_when_disconnected_label}>
                      Block when disconnected
                    </Cell.Label>
                    <Switch
                      isOn={this.props.blockWhenDisconnected}
                      onChange={this.props.setBlockWhenDisconnected}
                    />
                  </Cell.Container>
                  <Cell.Footer>
                    {
                      "Unless connected, always block all network traffic, even when you've disconnected or quit the app."
                    }
                  </Cell.Footer>

                  <View style={styles.advanced_settings__content}>
                    <Selector
                      title={'Network protocols'}
                      values={['Automatic', 'UDP', 'TCP']}
                      value={protocol}
                      onSelect={(selectedProtocol) => {
                        this.props.onUpdate(selectedProtocol, 'Automatic');
                      }}
                    />

                    <View style={styles.advanced_settings__cell_spacer} />

                    {portSelector}
                  </View>

                  <Cell.Container>
                    <Cell.Label>Mssfix</Cell.Label>
                    <Cell.InputFrame style={styles.advanced_settings__mssfix_frame}>
                      <Cell.Input
                        keyboardType={'numeric'}
                        maxLength={4}
                        placeholder={'Default'}
                        value={mssfixValue ? mssfixValue.toString() : ''}
                        style={mssfixStyle}
                        onChangeText={this.onMssfixChange}
                        onFocus={this.onMssfixFocus}
                        onBlur={this.onMssfixBlur}
                      />
                    </Cell.InputFrame>
                  </Cell.Container>
                  <Cell.Footer>
                    Set OpenVPN MSS value. Valid range: {MIN_MSSFIX_VALUE} - {MAX_MSSFIX_VALUE}.
                  </Cell.Footer>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  private createPortSelector() {
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

  private onMssfixChange = (mssfixString: string) => {
    const mssfix = mssfixString.replace(/[^0-9]/g, '');

    if (mssfix === '') {
      this.setState({ editedMssfix: undefined });
    } else {
      this.setState({ editedMssfix: parseInt(mssfix, 10) });
    }
  };

  private onMssfixFocus = () => {
    this.setState({ focusOnMssfix: true });
  };

  private onMssfixBlur = () => {
    this.setState({ focusOnMssfix: false });

    if (this.mssfixIsValid()) {
      this.props.setOpenVpnMssfix(this.state.editedMssfix);
      this.setState((state, _props) => ({ persistedMssfix: state.editedMssfix }));
    }
  };

  private mssfixIsValid(): boolean {
    const mssfix = this.state.editedMssfix;

    return mssfix === undefined || (mssfix >= MIN_MSSFIX_VALUE && mssfix <= MAX_MSSFIX_VALUE);
  }
}

interface ISelectorProps<T> {
  title: string;
  values: T[];
  value: T;
  onSelect: (value: T) => void;
}

interface ISelectorState<T> {
  hoveredButtonValue?: T;
}

class Selector<T> extends Component<ISelectorProps<T>, ISelectorState<T>> {
  public state: ISelectorState<T> = {};

  public render() {
    return (
      <View>
        <View style={styles.advanced_settings__section_title}>{this.props.title}</View>

        {this.props.values.map((value) => this.renderCell(value))}
      </View>
    );
  }

  private handleButtonHover = (value?: T) => {
    this.setState({ hoveredButtonValue: value });
  };

  private renderCell(value: T) {
    const selected = value === this.props.value;
    if (selected) {
      return this.renderSelectedCell(value);
    } else {
      return this.renderUnselectedCell(value);
    }
  }

  private renderSelectedCell(value: T) {
    return (
      <Button
        style={[
          styles.advanced_settings__cell,
          value === this.state.hoveredButtonValue
            ? [styles.advanced_settings__cell_selected_hover]
            : undefined,
        ]}
        onPress={() => this.props.onSelect(value)}
        onHoverStart={() => this.handleButtonHover(value)}
        onHoverEnd={() => this.handleButtonHover(undefined)}
        key={value.toString()}>
        <ImageView
          style={styles.advanced_settings__cell_icon}
          source="icon-tick"
          tintColor={colors.white}
        />
        <Text style={styles.advanced_settings__cell_label}>{value}</Text>
      </Button>
    );
  }

  private renderUnselectedCell(value: T) {
    return (
      <Button
        style={[
          styles.advanced_settings__cell_dimmed,
          value === this.state.hoveredButtonValue
            ? styles.advanced_settings__cell_hover
            : undefined,
        ]}
        onPress={() => this.props.onSelect(value)}
        onHoverStart={() => this.handleButtonHover(value)}
        onHoverEnd={() => this.handleButtonHover(undefined)}
        key={value.toString()}>
        <View style={styles.advanced_settings__cell_icon} />
        <Text style={styles.advanced_settings__cell_label}>{value}</Text>
      </Button>
    );
  }
}
