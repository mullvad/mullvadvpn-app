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
      fontSize: 13,
      fontWeight: '600',
      color: 'rgb(255, 255, 255, 0.4)',
    }),
    hovered: Styles.createTextStyle({
      color: 'rgb(255, 255, 255)',
    }),
  },
};

interface IProps {
  onToggle?: (isOpen: boolean) => void;
  defaultOpen?: boolean;
  children: string;
  style?: Types.ViewStyleRuleSet | Types.ViewStyleRuleSet[];
}

interface IState {
  isHovered: boolean;
  isOpen: boolean;
}

export default class ConnectionInfoDisclosure extends Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    this.state = {
      isHovered: false,
      isOpen: props.defaultOpen === true,
    };
  }

  public render() {
    const tintColor = this.state.isHovered ? 'rgb(255, 255, 255)' : 'rgb(255, 255, 255, 0.4)';

    return (
      <View
        style={[styles.container, this.props.style]}
        onMouseEnter={this.onMouseEnter}
        onMouseLeave={this.onMouseLeave}
        onPress={this.onToggle}>
        <Text
          style={[styles.caption.base, this.state.isHovered ? styles.caption.hovered : undefined]}>
          {this.props.children}
        </Text>
        <ImageView
          source={this.state.isOpen ? 'icon-chevron-up' : 'icon-chevron-down'}
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

  private onToggle = () => {
    this.setState(
      (state) => ({
        ...state,
        isOpen: !state.isOpen,
      }),
      () => {
        if (this.props.onToggle) {
          this.props.onToggle(this.state.isOpen);
        }
      },
    );
  };
}
