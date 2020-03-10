import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  advanced_settings: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  advanced_settings__container: Styles.createViewStyle({
    flex: 1,
  }),
  // plain CSS style
  advanced_settings__scrollview: {
    flex: 1,
  },
  advanced_settings__content: Styles.createViewStyle({
    flex: 0,
  }),
  advanced_settings__tunnel_protocol: Styles.createViewStyle({
    marginBottom: 24,
  }),
  advanced_settings__tunnel_protocol_selector: Styles.createViewStyle({
    marginBottom: 0,
  }),
  advanced_settings__wgkeys_cell: Styles.createViewStyle({
    marginBottom: 24,
  }),
  advanced_settings__wg_no_key: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    lineHeight: 20,
    color: colors.red,
    marginTop: 12,
    paddingHorizontal: 24,
  }),
  advanced_settings__cell_hover: Styles.createButtonStyle({
    backgroundColor: colors.blue80,
  }),
  advanced_settings__cell_selected_hover: Styles.createButtonStyle({
    backgroundColor: colors.green,
  }),
  advanced_settings__cell_icon_invisible: Styles.createViewStyle({
    opacity: 0,
  }),
  advanced_settings__cell_label: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    letterSpacing: -0.2,
    color: colors.white,
    flex: 0,
  }),
  advanced_settings__cell_footer_internet_warning_label: Styles.createTextStyle({
    marginTop: 4,
  }),
  advanced_settings__input_frame: Styles.createViewStyle({
    flex: 0,
  }),
  advanced_settings__block_when_disconnected_label: Styles.createTextStyle({
    letterSpacing: -0.5,
  }),
};
