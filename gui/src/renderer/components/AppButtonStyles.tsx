import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  red: Styles.createButtonStyle({
    backgroundColor: colors.red,
  }),
  redHover: Styles.createButtonStyle({
    backgroundColor: colors.red95,
  }),
  green: Styles.createButtonStyle({
    backgroundColor: colors.green,
  }),
  greenHover: Styles.createButtonStyle({
    backgroundColor: colors.green90,
  }),
  blue: Styles.createButtonStyle({
    backgroundColor: colors.blue80,
  }),
  blueHover: Styles.createButtonStyle({
    backgroundColor: colors.blue60,
  }),
  transparent: Styles.createButtonStyle({
    backgroundColor: colors.white20,
  }),
  transparentHover: Styles.createButtonStyle({
    backgroundColor: colors.white40,
  }),
  redTransparent: Styles.createButtonStyle({
    backgroundColor: colors.red40,
  }),
  redTransparentHover: Styles.createButtonStyle({
    backgroundColor: colors.red45,
  }),
  icon: Styles.createViewStyle({
    position: 'absolute',
    alignSelf: 'flex-end',
    right: 8,
    marginLeft: 8,
  }),
  common: Styles.createViewStyle({
    cursor: 'default',
    paddingTop: 9,
    paddingLeft: 9,
    paddingRight: 9,
    paddingBottom: 9,
    borderRadius: 4,
    flex: 1,
    flexDirection: 'column',
    alignContent: 'center',
    justifyContent: 'center',
  }),
  label: Styles.createTextStyle({
    alignSelf: 'center',
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    flex: 1,
    color: colors.white,
  }),
};
