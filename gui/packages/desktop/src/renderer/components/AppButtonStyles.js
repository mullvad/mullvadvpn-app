// @flow

import { Styles } from 'reactxp';
import { colors } from '../../config';

export default {
  red: Styles.createViewStyle({
    backgroundColor: colors.red,
  }),
  redHover: Styles.createViewStyle({
    backgroundColor: colors.red95,
  }),
  green: Styles.createViewStyle({
    backgroundColor: colors.green,
  }),
  greenHover: Styles.createViewStyle({
    backgroundColor: colors.green90,
  }),
  blue: Styles.createViewStyle({
    backgroundColor: colors.blue80,
  }),
  blueHover: Styles.createViewStyle({
    backgroundColor: colors.blue60,
  }),
  white80: Styles.createViewStyle({
    color: colors.white80,
  }),
  white: Styles.createViewStyle({
    color: colors.white,
  }),
  icon: Styles.createViewStyle({
    position: 'absolute',
    alignSelf: 'flex-end',
    right: 8,
    marginLeft: 8,
  }),
  iconTransparent: Styles.createViewStyle({
    position: 'absolute',
    alignSelf: 'flex-end',
    right: 42,
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
  }),
};
