import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';

const SIDE_BUTTON_WIDTH = 50;

const styles = {
  buttonRow: Styles.createViewStyle({
    flexDirection: 'row',
  }),
  mainButton: Styles.createViewStyle({
    flex: 1,
    borderTopRightRadius: 0,
    borderBottomRightRadius: 0,
  }),
  sideButton: Styles.createViewStyle({
    borderTopLeftRadius: 0,
    borderBottomLeftRadius: 0,
    width: SIDE_BUTTON_WIDTH,
    alignItems: 'center',
    marginLeft: 1,
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
      <View style={styles.buttonRow}>
        <MainButton textOffset={SIDE_BUTTON_WIDTH} style={styles.mainButton} />
        <SideButton style={styles.sideButton} />
      </View>
    );
  }
}
