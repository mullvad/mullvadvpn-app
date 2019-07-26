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
  advanced_settings__selector_section: Styles.createViewStyle({
    marginBottom: 24,
  }),
  advanced_settings__wgkeys_cell: Styles.createViewStyle({
    marginBottom: 24,
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
  advanced_settings__mssfix_input: Styles.createTextInputStyle({
    minWidth: 80,
  }),
  advanced_settings__mssfix_frame: Styles.createViewStyle({
    flex: 0,
  }),
  advanced_settings__mssfix_valid_value: Styles.createTextStyle({
    color: colors.white,
  }),
  advanced_settings__mssfix_invalid_value: Styles.createTextStyle({
    color: colors.red,
  }),
  advanced_settings__block_when_disconnected_label: Styles.createTextStyle({
    letterSpacing: -0.5,
  }),
};
