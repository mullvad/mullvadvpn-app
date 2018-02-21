// @flow

import React from 'react';
import ReactXP from 'reactxp';

const defaultStyle = ReactXP.Styles.createViewStyle({
  cursor: 'default',
});

export function Button(props: Object) {
  const { style, ...rest } = props;

  const concreteStyle = ReactXP.Styles.combine([defaultStyle, style]);

  return <ReactXP.Button style={concreteStyle} {...rest} />;
}
