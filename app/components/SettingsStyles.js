import { createViewStyles, createTextStyles } from '../lib/styles';

export default Object.assign(createViewStyles({
  settings: {
    backgroundColor: '#192E45',
    height: '100%'
  },
  settings__container:{
    flexDirection: 'column',
    height: '100%'
  },
  settings__header:{
    flexGrow: 0,
    flexShrink: 0,
    flexBasis: 'auto',
    paddingTop: 40,
    paddingRight: 24,
    paddingLeft: 24,
    paddingBottom: 24,
  },
  settings__content: {
    flexDirection: 'column',
    flex: 1,
    justifyContent: 'space-between',
  },
  settings__scrollview: {
    flexGrow: 1,
    flexShrink: 1,
    flexBasis: '100%',
  },
  settings__close: {
    position: 'absolute',
    top: 0,
    left: 12,
    borderWidth: 0,
    padding: 0,
    margin: 0,
    zIndex: 1, /* part of .settings__close covers the button */
    cursor: 'default',
  },
  settings__close_icon:{
    width: 24,
    height: 24,
    flex: 0,
    opacity: 0.6,
  },
  settings__icon_chevron:{
    marginLeft: 8,
    color: 'rgba(255, 255, 255, 0.8)',
    width: 7,
    height: 12,
  },
  settings__cell_spacer:{
    height: 24,
    flex: 0
  },
  settings__footer_button:{
    backgroundColor: 'rgba(208,2,27,1)',
    paddingTop: 7,
    paddingLeft: 12,
    paddingRight: 12,
    paddingBottom: 9,
    borderRadius: 4,
    alignItems: 'center',
  },
  settings__footer: {
    paddingTop: 24,
    paddingLeft: 24,
    paddingRight: 24,
    paddingBottom: 24,
  },
}), createTextStyles({
  settings__title:{
    fontFamily: 'DINPro',
    fontSize: 32,
    fontWeight: '900',
    lineHeight: 40,
    color: '#FFFFFF'
  },
  settings__cell_label:{
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    color: '#FFFFFF',
    flexGrow: 1,
    flexShrink: 0,
    flexBasis: 'auto',
  },
  settings__footer_button_label:{
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    color: 'rgba(255,255,255,0.8)'
  },
  settings__cell_subtext:{
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    color: 'rgba(255, 255, 255, 0.8)',
    flexGrow: 0,
    textAlign: 'right',
  },
  settings__account_paid_until_label__error:{
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    flexGrow: 0,
    textAlign: 'right',
    color: '#d0021b',
  },
}));
