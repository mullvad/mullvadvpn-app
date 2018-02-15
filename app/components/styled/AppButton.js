// @flow
import React from 'react';
import { Text, Component } from 'reactxp';
import { Button } from './Button';
import { Label } from './Label';
import { Icon } from './Icon';
import Img from '../Img';
import { colors } from '../../config';

import { createViewStyles, createTextStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    red:{
      backgroundColor: colors.red95,
    },
    redHover: {
      backgroundColor: colors.red,
    },
    green:{
      backgroundColor: colors.green,
    },
    greenHover:{
      backgroundColor: colors.green90,
    },
    blue:{
      backgroundColor: colors.blue80,
    },
    blueHover:{
      backgroundColor: colors.blue60,
    },
    transparent:{
      backgroundColor: colors.white20,
    },
    transparentHover:{
      backgroundColor: colors.white40,
    },
    white80:{
      color: colors.white80,
    },
    white: {
      color: colors.white,
    },
    common:{
      paddingTop: 7,
      paddingLeft: 12,
      paddingRight: 12,
      paddingBottom: 9,
      marginTop: 8,
      marginBottom: 8,
      marginLeft: 24,
      marginRight: 24,
      borderRadius: 4,
      flex: 1,
      flexDirection: 'column',
      alignContent: 'center',
      justifyContent: 'center',
    },
  }),
};

export default class BaseButton extends Component {
  props: {
    children: React.Element<*>,
    disabled: boolean,
  };

  state = { hovered: false };

  textStyle = () => this.state.hovered ? styles.white80 : styles.white;
  iconStyle = () => this.state.hovered ? styles.white80 : styles.white;
  backgroundStyle = () => this.state.hovered ? styles.white80 : styles.white;

  onHoverStart = () => !this.props.disabled ? this.setState({ hovered: true }) : null;
  onHoverEnd = () => !this.props.disabled ? this.setState({ hovered: false }) : null;
  render() {
    const { children, ...otherProps } = this.props;
    return (
      <Button style={[ styles.common, this.backgroundStyle() ]}
        onHoverStart={this.onHoverStart}
        onHoverEnd={this.onHoverEnd}
        {...otherProps}>
        {
          React.Children.map(children, (node) => {
            if (React.isValidElement(node)){
              let updatedProps = {};

              if(node.type.name === 'Label') {
                updatedProps = { style: this.textStyle() };
              }

              if(node.type.name === 'Icon') {
                updatedProps = { style: this.iconStyle() };
              }

              return React.cloneElement(node, updatedProps);
            } else {
              return <Label style={this.textStyle()}>{children}</Label>;
            }
          })
        }
      </Button>
    );
  }
}

export class RedButton extends BaseButton{
  backgroundStyle = () => this.state.hovered ? styles.redHover : styles.red;
}

export class GreenButton extends BaseButton{
  backgroundStyle = () => this.state.hovered ? styles.greenHover : styles.green;
}

export class BlueButton extends BaseButton{
  backgroundStyle = () => this.state.hovered ? styles.blueHover : styles.blue;
}

export class TransparentButton extends BaseButton{
  backgroundStyle = () => this.state.hovered ? styles.transparentHover : styles.transparent;
}