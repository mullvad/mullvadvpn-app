import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import * as Cell from './Cell';
import Selector from './Selector';

export const InputFrame = styled(Cell.InputFrame)({
  flex: 0,
});

export const TunnelProtocolSelector = (styled(Selector)({
  marginBottom: 0,
}) as unknown) as new <T>() => Selector<T>;

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
    color: colors.white,
    flex: 0,
  }),
  advanced_settings__cell_footer_internet_warning_label: Styles.createTextStyle({
    marginTop: 4,
  }),
};
