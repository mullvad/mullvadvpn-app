import * as React from 'react';
import { Component, Styles, Types, View } from 'reactxp';

interface IProps {
  arrowPosition?: number;
}

const containerStyle = Styles.createViewStyle({ flex: 1 });

export default class PlatformWindow extends Component<IProps> {
  public render() {
    return <View style={[containerStyle, this.platformStyle()]}>{this.props.children}</View>;
  }

  private platformStyle(): Types.ViewStyleRuleSet {
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

      return Styles.createViewStyle(
        {
          // @ts-ignore
          WebkitMask: webkitMask.join(','),
        },
        false,
      );
    } else {
      return undefined;
    }
  }
}
