// @flow
import * as React from 'react';
import { View, Component, Types } from 'reactxp';

type Props = {
  source: string,
  tintColor?: string,
  hoverStyle?: Types.ViewStyle,
  disabled?: boolean,
};

type State = { hovered: boolean };

export default class Img extends Component<Props, State> {
  state = { hovered: false };

  onHoverStart = () => (!this.props.disabled ? this.setState({ hovered: true }) : null);
  onHoverEnd = () => (!this.props.disabled ? this.setState({ hovered: false }) : null);

  getHoverStyle = () => (this.state.hovered ? this.props.hoverStyle || null : null);

  render() {
    const { source, style, onMouseEnter, onMouseLeave, ...otherProps } = this.props;
    const tintColor = this.props.tintColor;
    const url = './assets/images/' + source + '.svg';
    let image;

    if (tintColor) {
      image = (
        <div
          style={{
            WebkitMaskImage: `url('${url}')`,
            WebkitMaskRepeat: 'no-repeat',
            backgroundColor: tintColor,
            lineHeight: 0,
          }}>
          <img
            src={url}
            style={{
              visibility: 'hidden',
            }}
          />
        </div>
      );
    } else {
      image = <img src={url} />;
    }

    return (
      <View
        {...otherProps}
        onMouseEnter={onMouseEnter || this.onHoverStart}
        onMouseLeave={onMouseLeave || this.onHoverEnd}
        style={[style, this.getHoverStyle()]}>
        {image}
      </View>
    );
  }
}
