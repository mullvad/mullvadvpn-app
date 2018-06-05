// @flow

import { Styles } from 'reactxp';

type ExtractReturnType = (*) => Object;

export function createViewStyles<T: { [string]: Object }>(
  styles: T,
): $ObjMap<T, ExtractReturnType> {
  const viewStyles = {};
  for (const style of Object.keys(styles)) {
    viewStyles[style] = Styles.createViewStyle(styles[style]);
  }
  return viewStyles;
}

export function createTextStyles<T: { [string]: Object }>(
  styles: T,
): $ObjMap<T, ExtractReturnType> {
  const textStyles = {};
  for (const style of Object.keys(styles)) {
    textStyles[style] = Styles.createTextStyle(styles[style]);
  }
  return textStyles;
}
