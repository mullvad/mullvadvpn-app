// @flow
import React from 'react';
import { Text, Component } from 'reactxp';
import { Button } from './Button';
import Img from '../Img';

import { createViewStyles, createTextStyles } from '../../lib/styles';

const styles = {
  ...createViewStyles({
    cell:{
      backgroundColor: 'rgba(41,71,115,1)',
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
      backgroundColor: 'rgba(41,71,115,0.9)'
    },
    icon:{
      marginLeft: 8,
      width: 16,
      height: 16,
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      alignItems: 'flex-end',
      color: 'rgba(255, 255, 255, 0.8)',
    },
  }),
  ...createTextStyles({
    label:{
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      color: '#FFFFFF',
      flexGrow: 1,
      flexShrink: 0,
      flexBasis: 'auto',
    },
    subtext:{
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '800',
      color: 'rgba(255, 255, 255, 0.8)',
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
    onPress?: () => void,
    style?: string
  };

  state = { hovered: false };

  render() {
    const { style, tintColor, hoverStyle, text, subtext, subtextStyle, icon, iconStyle, onPress, ...otherProps } = this.props;

    return (
      <Button style={[ styles.cell, style, this.state.hovered ? [styles.hover, hoverStyle] : null ]}
        onPress={ onPress }
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
