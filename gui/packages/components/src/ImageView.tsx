import * as React from 'react';
import { Component, Types, View } from 'reactxp';

interface IProps {
  source: string;
  width?: number;
  height?: number;
  tintColor?: string;
  tintHoverColor?: string;
  disabled?: boolean;
  onPress?: (event: Types.SyntheticEvent) => void;
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

interface IState {
  hovered: boolean;
}

export default class ImageView extends Component<IProps, IState> {
  public state = { hovered: false };

  public render() {
    const { source, width, height, tintColor, tintHoverColor, ...otherProps } = this.props;
    const url = `../../assets/images/${source}.svg`;
    let image;

    const activeTintColor = (this.state.hovered && tintHoverColor) || tintColor;

    if (activeTintColor) {
      const maskWidth = typeof width === 'number' ? `${width}px` : 'auto';
      const maskHeight = typeof height === 'number' ? `${height}px` : 'auto';
      image = (
        <div
          style={{
            WebkitMaskImage: `url('${url}')`,
            WebkitMaskRepeat: 'no-repeat',
            WebkitMaskSize: `${maskWidth} ${maskHeight}`,
            backgroundColor: activeTintColor,
            lineHeight: 0,
          }}>
          <img
            src={url}
            width={width}
            height={height}
            style={{
              visibility: 'hidden',
            }}
          />
        </div>
      );
    } else {
      image = <img src={url} width={width} height={height} />;
    }

    return (
      <View {...otherProps} onMouseEnter={this.onHoverStart} onMouseLeave={this.onHoverEnd}>
        {image}
      </View>
    );
  }

  private onHoverStart = () => {
    if (!this.props.disabled) {
      this.setState({ hovered: true });
    }
  };

  private onHoverEnd = () => {
    if (!this.props.disabled) {
      this.setState({ hovered: false });
    }
  };
}
