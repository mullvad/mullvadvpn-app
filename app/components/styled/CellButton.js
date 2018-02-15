// @flow
import React from 'react';
import { Text, Component } from 'reactxp';
import { Button } from './Button';
import { Label } from './Label';
import { Icon } from './Icon';
import { SubText } from './SubText';
import Img from '../Img';
import { colors } from '../../config';

import { createViewStyles, createTextStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    cell:{
      paddingTop: 15,
      paddingBottom: 15,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      flex: 1,
      flexDirection: 'row',
      alignItems: 'center',
      alignContent: 'center',
      justifyContent: 'space-between'
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
    relative: {
      position: 'relative',
      alignSelf: 'center',
    },
  }),
};

export default class CellButton extends Component {
  props: {
    children: React.Element<*>,
    disabled: boolean,
  };

  state = { hovered: false };

  textStyle = () => this.state.hovered ? styles.white80 : styles.white;
  iconStyle = () => this.state.hovered ? styles.white80 : styles.white;
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
                updatedProps = { style: [this.textStyle(), node.props.style]};
              }

              if(node.type.name === 'Icon') {
                updatedProps = { style: [this.iconStyle(), styles.relative, node.props.style]};
              }

              if(node.type.name === 'SubText') {
                updatedProps = { style: [this.subtextStyle(), node.props.style]};
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
