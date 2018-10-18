// @flow

import * as React from 'react';
import { Button, Text, Component } from 'reactxp';
import { ImageView } from '@mullvad/components';
import styles from './AppButtonStyles';
import { colors } from '../../config';

export class Label extends Text {}

type Props = {
  children?: React.Node,
  style?: Object,
  disabled: boolean,
};

type State = {
  hovered: boolean,
};

class BaseButton extends Component<Props, State> {
  state = { hovered: false };

  backgroundStyle = () => {
    throw new Error('Implement backgroundStyle in subclasses.');
  };
  onHoverStart = () => (!this.props.disabled ? this.setState({ hovered: true }) : null);
  onHoverEnd = () => (!this.props.disabled ? this.setState({ hovered: false }) : null);

  render() {
    const { children, style, ...otherProps } = this.props;
    return (
      <Button
        {...otherProps}
        style={[styles.common, this.backgroundStyle(), style]}
        onHoverStart={this.onHoverStart}
        onHoverEnd={this.onHoverEnd}>
        {React.Children.map(children, (node) => {
          if (React.isValidElement(node)) {
            let updatedProps = {};

            if (node.type === Label) {
              updatedProps = { style: [styles.label] };
            }

            if (node.type === ImageView) {
              updatedProps = { tintColor: colors.white, style: [styles.icon] };
            }

            return React.cloneElement(node, updatedProps);
          } else {
            return <Label style={[styles.label]}>{children}</Label>;
          }
        })}
      </Button>
    );
  }
}

export class RedButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.redHover : styles.red);
}

export class GreenButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.greenHover : styles.green);
}

export class BlueButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.blueHover : styles.blue);
}

export class TransparentButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.transparentHover : styles.transparent);
}

export class RedTransparentButton extends BaseButton {
  backgroundStyle = () => (this.state.hovered ? styles.redTransparentHover : styles.redTransparent);
}
