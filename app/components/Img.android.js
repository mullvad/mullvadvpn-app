import React from 'react';
import { Image, Component } from 'reactxp';

export default class Img extends Component {
	render(): React.Element<*> {
    return (<Image style={ this.props.style } source={ this.props.source }/>);
  }
}