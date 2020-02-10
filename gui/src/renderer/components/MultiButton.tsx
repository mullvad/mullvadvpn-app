import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';
import { colors } from '../../config.json';

const SIDE_BUTTON_WIDTH = 50;

const styles = {
  button_row: Styles.createViewStyle({
    flexDirection: 'row',
  }),
  main_button: Styles.createViewStyle({
    flex: 1,
    borderTopRightRadius: 0,
    borderBottomRightRadius: 0,
  }),
  side_button: Styles.createViewStyle({
    borderTopLeftRadius: 0,
    borderBottomLeftRadius: 0,
    width: SIDE_BUTTON_WIDTH,
    alignItems: 'center',
    borderStyle: 'solid',
    borderColor: colors.darkRed,
    borderLeftWidth: 1,
  }),
};

interface IProps {
  mainButton: React.ComponentType<IMainButtonProps>;
  sideButton: React.ComponentType<ISideButtonProps>;
}

export interface IMainButtonProps {
  textOffset: number;
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

export interface ISideButtonProps {
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

export class MultiButton extends Component<IProps> {
  public render() {
    const { mainButton: MainButton, sideButton: SideButton } = this.props;

    return (
      <View style={styles.button_row}>
        <MainButton textOffset={SIDE_BUTTON_WIDTH} style={styles.main_button} />
        <SideButton style={styles.side_button} />
      </View>
    );
  }
}
