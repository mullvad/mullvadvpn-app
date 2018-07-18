// @flow

import { Styles, UserInterface } from 'reactxp';

const dimensions = UserInterface.measureWindow();
const styles = {
  animationDefaultStyle: Styles.createAnimatedViewStyle({
    position: 'absolute',
    width: dimensions.width,
    height: dimensions.height,
  }),
  allowPointerEventsStyle: Styles.createAnimatedViewStyle({
    pointerEvents: 'auto',
  }),
  transitionContainerStyle: Styles.createViewStyle({
    width: dimensions.width,
    height: dimensions.height,
  }),
};

export default () => {
  return styles;
};
