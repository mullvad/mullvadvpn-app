import * as React from 'react';
import { Component, Styles, View } from 'reactxp';

interface IProps {
  arrowPosition?: number;
}

export default class PlatformWindow extends Component<IProps> {
  public render() {
    let style;

    if (process.platform === 'darwin') {
      const arrowPosition = this.props.arrowPosition;
      let arrowPositionCss = '50%';

      if (typeof arrowPosition === 'number') {
        const arrowWidth = 30;
        const adjustedArrowPosition = arrowPosition - arrowWidth * 0.5;
        arrowPositionCss = `${adjustedArrowPosition}px`;
      }

      const webkitMask = [
        `url(../../assets/images/app-triangle.svg) ${arrowPositionCss} 0% no-repeat`,
        `url(../../assets/images/app-header-backdrop.svg) no-repeat`,
      ];

      // @ts-ignore
      style = Styles.createViewStyle({ WebkitMask: webkitMask.join(',') }, false);
    }

    return <View style={style}>{this.props.children}</View>;
  }
}
