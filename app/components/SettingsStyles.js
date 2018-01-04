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
    flex: 0,
    paddingTop: 12,
    paddingBottom: 12,
    paddingLeft: 24,
    paddingRight: 24,
    position: 'relative' /* anchor for close button */
  },
  settings__content:{
    flexDirection: 'column',
    flex: 1,
    justifyContent: 'space-between',
    height: '100%',
  },
  settings__close:{
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-start',
    marginTop: 0,
    marginLeft: 12,
  },
  settings__close_icon:{
    width: 24,
    height: 24,
    flex: 0,
    opacity: 0.6,
    marginRight: 8,
  },
  settings__cell:{
    backgroundColor: 'rgba(41,71,115,1)',
    paddingTop: 15,
    paddingBottom: 15,
    paddingLeft: 24,
    paddingRight: 24,
    marginBottom: 1,
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between'
  },
  settings__cell_disclosure:{
    marginLeft: 8,
    color: 'rgba(255, 255, 255, 0.8)',
  },
  settings__cell_spacer:{
    height: 24,
    flex: 0
  },
  settings__cell__active_hover:{
    backgroundColor: 'rgba(41,71,115,0.9)'
  },
  settings__cell_icon:{
    width: 16,
    height: 16,
    flexGrow: 0,
    flexShrink: 0,
    flexBasis: 'auto',
    alignItems: 'flex-end',
    color: 'rgba(255, 255, 255, 0.8)',
  },
  settings__account_paid_until_label__error:{
    color: '#d0021b',
  },
  settings__footer_button:{
    backgroundColor: 'rgba(208,2,27,1)',
    paddingTop: 7,
    paddingLeft: 12,
    paddingRight: 12,
    paddingBottom: 9,
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
    width: '100%',
  },
  settings__footer:{
    width: '100%',
    justifyContent: 'center',
    alignItems: 'center',
    paddingTop: 24,
    paddingLeft: 24,
    paddingRight: 24,
    paddingBottom: 24 * 2, // Not entirely sure why I need to double the padding here :/
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
  settings__account_paid_until_label_container :{
    flexGrow: 0,
    textAlign: 'right',
  },
  settings__account_paid_until_label:{
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    color: 'rgba(255, 255, 255, 0.8)',
  },
}));
