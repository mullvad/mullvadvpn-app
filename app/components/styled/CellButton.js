// @flow

import * as React from 'react';
import { Text, Component, Types } from 'reactxp';
import { Button } from './Button';
import { colors } from '../../config';

import { createViewStyles, createTextStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    cell:{
      backgroundColor: colors.blue80,
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
      color: colors.white60,
      marginLeft: 8,
    },
  }),
  ...createTextStyles({
    label:{
      color: colors.white,
      alignSelf: 'center',
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      flex: 1,
      marginLeft: 8,
    },
    subtext:{
      color: colors.white60,
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


type CellButtonProps = {
  children?: React.Node,
  disabled?: boolean,
  cellHoverStyle?: Types.ViewStyle,
  style?: Types.ViewStyle
};

type State = { hovered: boolean };

export default class CellButton extends Component<CellButtonProps, State> {
  state = { hovered: false };

  textStyle = (cellHoverStyle?: Types.ViewStyle) => this.state.hovered ? cellHoverStyle : null;
  iconStyle = (cellHoverStyle?: Types.ViewStyle) => this.state.hovered ? cellHoverStyle : null;
  subtextStyle = (cellHoverStyle?: Types.ViewStyle) => this.state.hovered ? cellHoverStyle : null;
  backgroundStyle = (cellHoverStyle?: Types.ViewStyle) => this.state.hovered ? cellHoverStyle || styles.blueHover : null;

  onHoverStart = () => !this.props.disabled ? this.setState({ hovered: true }) : null;
  onHoverEnd = () => !this.props.disabled ? this.setState({ hovered: false }) : null;

  render() {
    const { children, style, cellHoverStyle, ...otherProps } = this.props;
    return (
      <Button style={[ styles.cell, style, this.backgroundStyle(cellHoverStyle) ]}
        onHoverStart={this.onHoverStart}
        onHoverEnd={this.onHoverEnd}
        {...otherProps}>
        {
          React.Children.map(children, (node) => {
            if (React.isValidElement(node)) {
              let updatedProps = {};

              if(node.type.name === 'Label') {
                updatedProps = { style: [styles.label, node.props.style, this.textStyle(node.props.cellHoverStyle)]};
              }

              if(node.type.name === 'Img') {
                updatedProps = { tintColor:'currentColor', style: [styles.icon, node.props.style, this.iconStyle(node.props.cellHoverStyle)]};
              }

              if(node.type.name === 'SubText') {
                updatedProps = { style: [styles.subtext, node.props.style, this.subtextStyle(node.props.cellHoverStyle)]};
              }

              return React.cloneElement(node, updatedProps);
            } else if (node){
              return <Label style={[styles.label, this.textStyle()]}>{children}</Label>;
            }
          })
        }
      </Button>
    );
  }
}
