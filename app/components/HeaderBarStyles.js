// @flow
import { Styles } from 'reactxp';

const styles = {
  headerbar:
  Styles.createViewStyle({
    paddingTop: 12,
    paddingBottom: 12,
    paddingLeft: 12,
    paddingRight: 12,
    backgroundColor: '#294D73',
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  }),
  headerbar__hidden:
  Styles.createViewStyle({
    paddingTop: 24,
    paddingBottom: 0,
    paddingLeft: 0,
    paddingRight: 0,
  }),
  headerbar__darwin: Styles.createViewStyle({
    paddingTop: 24,
  }),
  headerbar__style_defaultDark:
  Styles.createViewStyle({
    backgroundColor: '#192E45',
  }),
  headerbar__style_error:
  Styles.createViewStyle({
    backgroundColor: '#D0021B',
  }),
  headerbar__style_success:
  Styles.createViewStyle({
    backgroundColor: '#44AD4D',
  }),
  headerbar__container:
  Styles.createViewStyle({
    display: 'flex',
    flexDirection: 'row',
    alignItems: 'center',
  }),
  headerbar__title:
  Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 24,
    fontWeight: '900',
    lineHeight: 30,
    letterSpacing: -0.5,
    color: 'rgba(255,255,255,0.6)',
    marginLeft: 8,
  }),
  headerbar__logo:
  Styles.createViewStyle({
    height: 50,
    width: 50,
  }),
  headerbar__settings:
  Styles.createViewStyle({
    width: 24,
    height: 24,
    backgroundColor: 'transparent',
    marginLeft: -6, //Because of button.css, when removed remove this
    marginTop: -1, //Because of button.css, when removed remove this
  })
};

module.exports = styles;
