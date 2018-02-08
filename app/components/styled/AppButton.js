// @flow
import React from 'react';
import { View, Text, Component } from 'reactxp';
import { Button } from './Button';
import Img from '../Img';
import { colors } from '../../config';

import { createViewStyles, createTextStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    cell:{
      backgroundColor: colors.blue80,
      paddingTop: 7,
      paddingLeft: 12,
      paddingRight: 12,
      paddingBottom: 9,
      borderRadius: 4,
      flex: 1,
      flexDirection: 'row',
      alignItems: 'center',
      alignContent: 'center',
      justifyContent: 'space-between',
    },
    hover:{
      backgroundColor: colors.blue60,
    },
    icon:{
      marginLeft: 8,
      width: 0,
      height: 0,
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      alignItems: 'flex-end',
      color: colors.white80,
    },
  }),
  ...createTextStyles({
    label:{
      alignItems: 'center',
      alignContent: 'center',
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      color: colors.white80,
    },
    labelHover: {
      color: colors.white,
    },
  })
};

export default class AppButton extends Component {
  props: {
    icon?: string,
    iconStyle?: string,
    hoverStyle?: string,
    text: string,
    textHoverStyle?: string,
    tintColor?: string,
    style?: string,
    disabled?: boolean,
  };

  state = { hovered: false };

  render() {
    const { style, tintColor, hoverStyle, text, textHoverStyle, icon, iconStyle, disabled, ...otherProps } = this.props;

    return (
      <Button style={[ styles.cell, style, this.state.hovered ? [styles.hover, hoverStyle] : null ]}
        onHoverStart={() => !disabled ? this.setState({ hovered: true }) : null }
        onHoverEnd={() => !disabled ? this.setState({ hovered: false }) : null }
        disabled={ disabled }
        {...otherProps}>

        <View style={[ styles.icon, iconStyle ]}/>

        <Text style={[ styles.label, this.state.hovered ? [styles.labelHover, textHoverStyle] : null ]}>{ text }</Text>

        {icon ? <Img style={[ styles.icon, iconStyle ]}
          source={ icon }
          tintColor={ tintColor }/> : <View style={[ styles.icon, iconStyle ]}/> }

      </Button>
    );
  }
}
