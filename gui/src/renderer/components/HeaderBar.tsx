import * as React from 'react';
import { Button, Component, Styles, Text, Types, View } from 'reactxp';
import { messages } from '../../shared/gettext';
import ImageView from './ImageView';

export enum HeaderBarStyle {
  default = 'default',
  defaultDark = 'defaultDark',
  error = 'error',
  success = 'success',
}

export interface IHeaderBarProps {
  barStyle: HeaderBarStyle;
  style?: Types.ViewStyleRuleSet;
}

const headerBarStyles = {
  container: {
    base: Styles.createViewStyle({
      paddingTop: 12,
      paddingBottom: 12,
      paddingLeft: 12,
      paddingRight: 12,
    }),
    darwin: Styles.createViewStyle({
      paddingTop: 24,
    }),
  },
  content: Styles.createViewStyle({
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-end',
    // the size of "brand" logo
    minHeight: 51,
  }),
  barStyle: {
    default: Styles.createViewStyle({
      backgroundColor: 'rgb(41, 77, 115)', // colors.blue
    }),
    defaultDark: Styles.createViewStyle({
      backgroundColor: 'rgb(25, 46, 69)', // colors.darkBlue
    }),
    error: Styles.createViewStyle({
      backgroundColor: 'rgb(227, 64, 57)', // colors.red
    }),
    success: Styles.createViewStyle({
      backgroundColor: 'rgb(68, 173, 77)', // colors.green
    }),
  },
};

export default class HeaderBar extends Component<IHeaderBarProps> {
  public static defaultProps: IHeaderBarProps = {
    barStyle: HeaderBarStyle.default,
  };

  public render() {
    const style = [
      headerBarStyles.container.base,
      process.platform === 'darwin' ? headerBarStyles.container.darwin : undefined,
      headerBarStyles.barStyle[this.props.barStyle],
      this.props.style,
    ];

    return (
      <View style={style}>
        <View style={headerBarStyles.content}>{this.props.children}</View>
      </View>
    );
  }
}

const brandStyles = {
  container: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  }),
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 24,
    fontWeight: '900',
    lineHeight: 30,
    letterSpacing: -0.5,
    color: 'rgba(255, 255, 255, 0.6)', // colors.white60
    marginLeft: 8,
  }),
};

export class Brand extends Component {
  public render() {
    return (
      <View style={brandStyles.container}>
        <ImageView width={50} height={50} source="logo-icon" />
        <Text style={brandStyles.title}>{messages.pgettext('headerbar', 'MULLVAD VPN')}</Text>
      </View>
    );
  }
}

interface ISettingsButtonProps {
  onPress?: () => void;
}

const settingsBarButtonStyle = Styles.createButtonStyle({
  cursor: 'default',
  padding: 0,
  marginLeft: 8,
});

export class SettingsBarButton extends Component<ISettingsButtonProps> {
  public render() {
    return (
      <Button style={settingsBarButtonStyle} onPress={this.props.onPress}>
        <ImageView
          height={24}
          width={24}
          source="icon-settings"
          tintColor={'rgba(255, 255, 255, 0.6)'}
          tintHoverColor={'rgba(255, 255, 255, 0.8)'}
        />
      </Button>
    );
  }
}
