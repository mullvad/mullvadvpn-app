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
    backgroundColor: colors.red60,
  }),
  redTransparentHover: Styles.createButtonStyle({
    backgroundColor: colors.red80,
  }),
  common: Styles.createViewStyle({
    cursor: 'default',
    borderRadius: 4,
  }),
  content: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    padding: 9,
  }),
  labelContainer: Styles.createViewStyle({
    flex: 1,
  }),
  label: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    flex: 1,
    color: colors.white,
    textAlign: 'center',
  }),
};
