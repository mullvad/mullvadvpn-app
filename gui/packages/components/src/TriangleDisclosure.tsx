import * as React from 'react';
import { Component, Styles, Text, Types, View } from 'reactxp';
import ImageView from './ImageView';

const styles = {
  container: Styles.createViewStyle({
    flexDirection: 'row',
    alignItems: 'center',
    flex: 0,
  }),
  caption: {
    base: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      marginLeft: 4,
      color: 'rgb(255, 255, 255, 0.4)',
    }),
    hovered: Styles.createTextStyle({
      color: 'rgb(255, 255, 255)',
    }),
  },
  disclosureOpened: Styles.createViewStyle({
    transform: [
      {
        rotate: '90',
      },
    ],
  }),
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

export default class TriangleDisclosure extends Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

    this.state = {
      isHovered: false,
      isOpen: !!props.defaultOpen,
    };
  }

  public render() {
    const tintColor =
      this.state.isHovered || this.state.isOpen ? 'rgb(255, 255, 255)' : 'rgb(255, 255, 255, 0.4)';

    return (
      <View
        style={[styles.container, this.props.style]}
        onMouseEnter={this.onMouseEnter}
        onMouseLeave={this.onMouseLeave}
        onPress={this.onToggle}>
        <ImageView
          style={[this.state.isOpen ? styles.disclosureOpened : undefined]}
          source={'icon-triangle-disclosure'}
          width={7}
          tintColor={tintColor}
        />
        <Text
          style={[
            styles.caption.base,
            this.state.isHovered || this.state.isOpen ? styles.caption.hovered : undefined,
          ]}>
          {this.props.children}
        </Text>
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
