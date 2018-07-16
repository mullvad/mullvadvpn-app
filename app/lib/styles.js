// @flow

import * as RX from 'reactxp';

type ExtractViewReturnType = <V>(V) => Object;
type ExtractTextReturnType = <V>(V) => Object;

export function createViewStyles<T: { [string]: Object }>(
  styles: T,
): $ObjMap<T, ExtractViewReturnType> {
  const viewStyles = {};
  for (const style of Object.keys(styles)) {
    viewStyles[style] = RX.Styles.createViewStyle(styles[style]);
  }
  return viewStyles;
}

export function createTextStyles<T: { [string]: Object }>(
  styles: T,
): $ObjMap<T, ExtractTextReturnType> {
  const textStyles = {};
  for (const style of Object.keys(styles)) {
    textStyles[style] = RX.Styles.createTextStyle(styles[style]);
  }
  return textStyles;
}
