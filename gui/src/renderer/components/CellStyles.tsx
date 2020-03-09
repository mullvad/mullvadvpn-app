import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  cellButton: {
    base: Styles.createButtonStyle({
      backgroundColor: colors.blue,
      paddingVertical: 0,
      paddingHorizontal: 16,
      marginBottom: 1,
      flex: 1,
      flexDirection: 'row',
      alignItems: 'center',
      alignContent: 'center',
      cursor: 'default',
    }),
    section: Styles.createButtonStyle({
      backgroundColor: colors.blue40,
    }),
    hover: Styles.createButtonStyle({
      backgroundColor: colors.blue80,
    }),
    selected: Styles.createViewStyle({
      backgroundColor: colors.green,
    }),
  },
  cellContainer: Styles.createViewStyle({
    backgroundColor: colors.blue,
    flexDirection: 'row',
    alignItems: 'center',
    paddingLeft: 16,
    paddingRight: 12,
  }),
  footer: {
    container: Styles.createViewStyle({
      paddingTop: 8,
      paddingRight: 24,
      paddingBottom: 24,
      paddingLeft: 24,
    }),
    text: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: colors.white80,
    }),
    boldText: Styles.createTextStyle({
      fontWeight: '900',
    }),
  },
  label: {
    container: Styles.createViewStyle({
      marginLeft: 8,
      marginTop: 14,
      marginBottom: 14,
      flex: 1,
    }),
    text: Styles.createTextStyle({
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      letterSpacing: -0.2,
      color: colors.white,
    }),
  },
  switch: Styles.createViewStyle({
    flex: 0,
  }),
  input: {
    frame: Styles.createViewStyle({
      flexGrow: 0,
      backgroundColor: 'rgba(255,255,255,0.1)',
      borderRadius: 4,
      padding: 4,
    }),
    text: Styles.createTextInputStyle({
      color: colors.white,
      backgroundColor: 'transparent',
      fontFamily: 'Open Sans',
      fontSize: 20,
      fontWeight: '600',
      lineHeight: 26,
      textAlign: 'right',
      padding: 0,
      minWidth: 80,
    }),
    validValue: Styles.createTextStyle({
      color: colors.white,
    }),
    invalidValue: Styles.createTextStyle({
      color: colors.red,
    }),
  },
  autoSizingInputContainer: {
    measuringView: Styles.createViewStyle({
      position: 'absolute',
      opacity: 0,
    }),
    measureText: Styles.createTextStyle({
      width: undefined,
      flexBasis: undefined,
    }),
  },
  icon: Styles.createViewStyle({
    marginLeft: 8,
  }),
  subtext: Styles.createTextStyle({
    color: colors.white60,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    flex: -1,
    textAlign: 'right',
    marginLeft: 8,
  }),
  sectionTitle: Styles.createTextStyle({
    backgroundColor: colors.blue,
    paddingVertical: 14,
    paddingHorizontal: 24,
    marginBottom: 1,
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    color: colors.white,
  }),
};
