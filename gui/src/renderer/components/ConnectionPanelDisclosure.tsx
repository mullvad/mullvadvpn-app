import * as React from 'react';
import { Component, Styles, Text, Types, View } from 'reactxp';
import ImageView from './ImageView';

const styles = {
  container: Styles.createViewStyle({
    flexDirection: 'row',
    alignItems: 'center',
  }),
  caption: {
    base: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 15,
      fontWeight: '600',
      lineHeight: 20,
      color: 'rgb(255, 255, 255, 0.4)',
    }),
    hovered: Styles.createTextStyle({
      color: 'rgb(255, 255, 255)',
    }),
  },
};

interface IProps {
  pointsUp: boolean;
  onToggle?: () => void;
  children: React.ReactText;
  style?: Types.ViewStyleRuleSet | Types.ViewStyleRuleSet[];
}

interface IState {
  isHovered: boolean;
}

export default class ConnectionPanelDisclosure extends Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    this.state = {
      isHovered: false,
    };
  }

  public render() {
    const tintColor = this.state.isHovered ? 'rgb(255, 255, 255)' : 'rgb(255, 255, 255, 0.4)';
    const textHoverStyle =
      this.props.pointsUp || this.state.isHovered ? styles.caption.hovered : undefined;

    return (
      <View
        style={[styles.container, this.props.style]}
        onMouseEnter={this.onMouseEnter}
        onMouseLeave={this.onMouseLeave}
        onPress={this.props.onToggle}>
        <Text style={[styles.caption.base, textHoverStyle]}>{this.props.children}</Text>
        <ImageView
          source={this.props.pointsUp ? 'icon-chevron-up' : 'icon-chevron-down'}
          width={24}
          height={24}
          tintColor={tintColor}
        />
      </View>
    );
  }

  private onMouseEnter = () => {
    this.setState({ isHovered: true });
  };

  private onMouseLeave = () => {
    this.setState({ isHovered: false });
  };
}
