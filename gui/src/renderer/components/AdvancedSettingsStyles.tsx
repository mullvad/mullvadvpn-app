import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import * as Cell from './Cell';
import { NavigationScrollbars } from './NavigationBar';
import Selector from './Selector';

export const InputFrame = styled(Cell.InputFrame)({
  flex: 0,
});

export const TunnelProtocolSelector = (styled(Selector)({
  marginBottom: 0,
}) as unknown) as new <T>() => Selector<T>;

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export default {
  advanced_settings: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  advanced_settings__container: Styles.createViewStyle({
    flex: 1,
  }),
  advanced_settings__content: Styles.createViewStyle({
    flex: 0,
  }),
  advanced_settings__cell_bottom_margin: Styles.createViewStyle({
    marginBottom: 20,
  }),
  advanced_settings__wg_no_key: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    lineHeight: 20,
    color: colors.red,
    marginTop: 12,
    paddingHorizontal: 22,
  }),
};
