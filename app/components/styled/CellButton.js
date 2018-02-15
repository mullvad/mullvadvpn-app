// @flow
import React from 'react';
import { Text, Component } from 'reactxp';
import { Button } from './Button';
import { colors } from '../../config';

import { createViewStyles, createTextStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    cell:{
      paddingTop: 14,
      paddingBottom: 14,
      paddingLeft: 16,
      paddingRight: 16,
      marginBottom: 1,
      flex: 1,
      flexDirection: 'row',
      alignItems: 'center',
      alignContent: 'center',
    },
    blue:{
      backgroundColor: colors.blue80,
    },
    blueHover:{
      backgroundColor: colors.blue60,
    },
    white40:{
      color: colors.white40,
    },
    white60:{
      color: colors.white60,
    },
    white80:{
      color: colors.white80,
    },
    white: {
      color: colors.white,
    },
    icon: {
      marginLeft: 8,
    },
  }),
  ...createTextStyles({
    label:{
      alignSelf: 'center',
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      flex: 1,
      marginLeft: 8,
    },
    subtext:{
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '800',
      flex: 0,
      textAlign: 'right',
    },
  }),
};

export class SubText extends Text {}
export class Label extends Text {}

export default class CellButton extends Component {
  props: {
    children: Array<React.Element<*>> | React.Element<*>,
    disabled: boolean,
  };

  state = { hovered: false };

  textStyle = () => this.state.hovered ? styles.white80 : styles.white;
  iconStyle = () => this.state.hovered ? styles.white40 : styles.white60;
  subtextStyle = () => this.state.hovered ? styles.white40 : styles.white60;
  backgroundStyle = () => this.state.hovered ? styles.blueHover : styles.blue;

  onHoverStart = () => !this.props.disabled ? this.setState({ hovered: true }) : null;
  onHoverEnd = () => !this.props.disabled ? this.setState({ hovered: false }) : null;

  render() {
    const { children, ...otherProps } = this.props;
    return (
      <Button style={[ styles.cell, this.backgroundStyle() ]}
        onHoverStart={this.onHoverStart}
        onHoverEnd={this.onHoverEnd}
        {...otherProps}>
        {
          React.Children.map(children, (node) => {
            if (React.isValidElement(node)){
              let updatedProps = {};

              if(node.type.name === 'Label') {
                updatedProps = { style: [styles.label, this.textStyle(), node.props.style]};
              }

              if(node.type.name === 'Img') {
                updatedProps = { tintColor:'currentColor', style: [styles.icon, this.iconStyle(), node.props.style]};
              }

              if(node.type.name === 'SubText') {
                updatedProps = { style: [styles.subtext, this.subtextStyle(), node.props.style]};
              }

              return React.cloneElement(node, updatedProps);
            } else {
              return <Label style={[styles.label, this.textStyle()]}>{children}</Label>;
            }
          })
        }
      </Button>
    );
  }
}
