// @flow

import React from 'react';
import ReactXP from 'reactxp';
const defaultStyle = {
};
export function Button(props: *) {
  const { style, cursor, ...rest } = props;

  const concreteStyle = Object.assign({}, defaultStyle, style);

  // Can be removed when we upgrade to ReactXP 0.51
  const concreteCursor = cursor || 'default';

  return <ReactXP.Button style={concreteStyle} cursor={concreteCursor} {...rest} />
}
