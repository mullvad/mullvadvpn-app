// @flow

import { Styles } from 'reactxp';
import { colors } from '../../config';

export default {
  connect: Styles.createViewStyle({
    flex: 1,
  }),
  map: Styles.createViewStyle({
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    zIndex: 0,
    height: '100%',
    width: '100%',
  }),
  container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
    position: 'relative' /* need this for z-index to work to cover map */,
    zIndex: 1,
  }),
  footer: Styles.createViewStyle({
    flex: 0,
    paddingBottom: 16,
    paddingLeft: 24,
    paddingRight: 24,
  }),
  blocking_container: Styles.createViewStyle({
    width: '100%',
    position: 'absolute',
  }),
  blocking_icon: Styles.createViewStyle({
    width: 10,
    height: 10,
    flex: 0,
    display: 'flex',
    borderRadius: 5,
    marginTop: 4,
    marginRight: 8,
    backgroundColor: colors.red,
  }),
  status: Styles.createViewStyle({
    paddingTop: 0,
    paddingLeft: 24,
    paddingRight: 24,
    paddingBottom: 0,
    marginTop: 186,
    flex: 1,
  }),
  status_icon: Styles.createViewStyle({
    position: 'absolute',
    alignSelf: 'center',
    width: 60,
    height: 60,
    marginTop: 94,
  }),
  switch_location_button: Styles.createViewStyle({
    marginBottom: 16,
  }),

  blocking_message: Styles.createTextStyle({
    display: 'flex',
    flexDirection: 'row',
    fontFamily: 'Open Sans',
    fontSize: 12,
    fontWeight: '800',
    lineHeight: 17,
    paddingTop: 8,
    paddingLeft: 20,
    paddingRight: 20,
    paddingBottom: 8,
    color: colors.white60,
    backgroundColor: colors.blue,
  }),
  server_label: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 32,
    fontWeight: '900',
    lineHeight: 44,
    letterSpacing: -0.7,
    color: colors.white,
    marginBottom: 7,
    flex: 0,
  }),
  error_title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 32,
    fontWeight: '900',
    lineHeight: 40,
    color: colors.white,
    marginBottom: 8,
  }),
  error_message: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    color: colors.white,
    marginBottom: 24,
  }),
  status_security: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '800',
    lineHeight: 22,
    marginBottom: 4,
    color: colors.white,
  }),
  status_security__secure: Styles.createTextStyle({
    color: colors.green,
  }),
  status_security__unsecured: Styles.createTextStyle({
    color: colors.red,
  }),
  status_ipaddress: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '800',
    color: colors.white,
  }),
  status_ipaddress__invisible: Styles.createTextStyle({
    opacity: 0,
  }),
  status_location: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 38,
    fontWeight: '900',
    lineHeight: 40,
    overflow: 'hidden',
    letterSpacing: -0.9,
    color: colors.white,
    marginBottom: 4,
  }),
};
