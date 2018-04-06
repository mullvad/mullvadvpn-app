// @flow
import { Styles, UserInterface } from 'reactxp';
import { MobileAppBridge } from 'NativeModules';

const dimensions = UserInterface.measureWindow();
let menuBarHeight;

/*MobileAppBridge.getHeight().then(_response => {height = _response}).catch(e => {
  log.error('Failed getting menuBarHeight:', e);
});
*/
MobileAppBridge.getMenuBarHeight().then(_response => {menuBarHeight = _response}).catch(e => {
  log.error('Failed getting menuBarHeight:', e);
});


export default getStyles = () => {
  return {
    animationDefaultStyle: Styles.createAnimatedViewStyle({
      position: 'absolute',
      width: dimensions.width,
      height: dimensions.height - menuBarHeight + 24,
    }, false),
    allowPointerEventsStyle: null,
    transitionContainerStyle: Styles.createViewStyle({
      width: dimensions.width,
      height: dimensions.height - menuBarHeight + 24, //TODO: Remove ugly hack since it seems that at least my LG is hard to find the real display area ... Probably needs to be fixed for some versions or models
    }, false)
  }
};
