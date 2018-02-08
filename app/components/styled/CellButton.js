// @flow
import React from 'react';
import { Text, Component } from 'reactxp';
import { Button } from './Button';
import Img from '../Img';
import { colors } from '../../config';

import { createViewStyles, createTextStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    cell:{
      backgroundColor: colors.blue,
      paddingTop: 15,
      paddingBottom: 15,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      flex: 1,
      flexDirection: 'row',
      alignItems: 'center',
      justifyContent: 'space-between'
    },
    hover:{
      backgroundColor: colors.blue80
    },
    icon:{
      marginLeft: 8,
      width: 16,
      height: 16,
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      alignItems: 'flex-end',
      color: colors.white80,
    },
  }),
  ...createTextStyles({
    label:{
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      color: colors.white,
      flexGrow: 1,
      flexShrink: 0,
      flexBasis: 'auto',
    },
    subtext:{
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '800',
      color: colors.white80,
      flexGrow: 0,
      textAlign: 'right',
    },
  })
};

export default class CellButton extends Component {
  props: {
    icon?: string,
    iconStyle?: string,
    hoverStyle?: string,
    subtextStyle?: string,
    text: string,
    subtext?: string,
    tintColor?: string,
    style?: string
  };

  state = { hovered: false };

  render() {
    const { style, tintColor, hoverStyle, text, subtext, subtextStyle, icon, iconStyle, ...otherProps } = this.props;

    return (
      <Button style={[ styles.cell, style, this.state.hovered ? [styles.hover, hoverStyle] : null ]}
        onHoverStart={() => this.setState({ hovered: true })}
        onHoverEnd={() => this.setState({ hovered: false })}
        {...otherProps}>

        <Text style={ styles.label }>{ text }</Text>

        { subtext ? <Text style={[ styles.subtext, subtextStyle ]}>{ subtext }</Text> : null }

        <Img style={[ styles.icon, iconStyle ]}
          source={ icon }
          tintColor={ tintColor }/>

      </Button>
    );
  }
}
