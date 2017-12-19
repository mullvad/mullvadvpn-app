import { Styles } from 'reactxp';

export function createViewStyles(styles: { [string]: Object }) {
  const viewStyles = {};
  for (const style of Object.keys(styles)) {
    viewStyles[style] = Styles.createViewStyle(styles[style]);
  }
  return viewStyles;
}

export function createTextStyles(styles: { [string]: Object }) {
  const textStyles = {};
  for (const style of Object.keys(styles)) {
    textStyles[style] = Styles.createTextStyle(styles[style]);
  }
  return textStyles;
}