import React from 'react';
import { View, Component } from 'reactxp';

export default class Img extends Component {
	render(): React.Element<*> {

	const url = "./assets/images/" + this.props.source + ".svg";

	const style = this.props.style;

    return (<View style={ style }> <img src={ url } /> </View>);
  }
}