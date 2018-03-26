// @flow
import * as React from 'react';
import { View, Component, Styles } from 'reactxp';

type ImgProps = {
    source: string,
    tintColor?: string,
    hoverColor?: string,
    disabled?: boolean,
  };

type State = { hovered: boolean };

export default class Img extends Component<ImgProps, State> {

  state = { hovered: false };

  hoverColorStyle = Styles.createViewStyle({color: this.props.hoverColor}, false);

  onHoverStart = () => !this.props.disabled ? this.setState({ hovered: true }) : null;
  onHoverEnd = () => !this.props.disabled ? this.setState({ hovered: false }) : null;

  getHoverStyle = () => (this.state.hovered && this.props.hoverColor) ? this.hoverColorStyle : null;

  render() {
    const { source, disabled, style, onMouseEnter, onMouseLeave, ...otherProps } = this.props;
    const tintColor = this.props.tintColor;
    const url = './assets/images/' + source + '.svg';
    let image;

    if(tintColor) {
      image = (
        <div style={{
          WebkitMaskImage: `url('${url}')`,
          WebkitMaskRepeat: 'no-repeat',
          backgroundColor: tintColor,
          lineHeight: 0,
        }}>
          <img src={ url } style={{
            visibility: 'hidden',
          }} />
        </div>
      );
    } else {
      image = (
        <img src={ url } />
      );
    }

    return (
      <View { ...otherProps }
        onMouseEnter={onMouseEnter || this.onHoverStart}
        onMouseLeave={onMouseLeave || this.onHoverEnd}
        style={[style, this.getHoverStyle()]}>
        { image }
      </View>);
  }
}
