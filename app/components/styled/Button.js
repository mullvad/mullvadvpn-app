// @flow

import React from 'react';
import ReactXP from 'reactxp';

const defaultStyle = ReactXP.Styles.createViewStyle({
  cursor: 'default',
});

export function Button(props: *) {
  const { style, cursor, ...rest } = props;

  const stylePropArray = Array.isArray(props.style)
    ? props.style
    : [props.style];

  const concreteStyle = ReactXP.Styles.combine([defaultStyle, ...stylePropArray]);

  // Can be removed when we upgrade to ReactXP 0.51
  const concreteCursor = cursor || concreteStyle.cursor || 'default';

  return <ReactXP.Button style={concreteStyle} cursor={concreteCursor} {...rest} />;
}
