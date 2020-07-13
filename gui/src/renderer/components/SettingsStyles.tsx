import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import * as Cell from './Cell';

export const OutOfTimeSubText = styled(Cell.SubText)((props: { isOutOfTime: boolean }) => ({
  color: props.isOutOfTime ? colors.red : undefined,
}));

export const CellIcon = styled(Cell.UntintedIcon)({
  marginRight: '8px',
});

export default {
  settings: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  content: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
    justifyContent: 'space-between',
    overflow: 'visible',
  }),
  // plain CSS style
  scrollview: {
    flex: 1,
  },
  cellSpacer: Styles.createViewStyle({
    height: 20,
    flex: 0,
  }),
  cellFooter: Styles.createViewStyle({
    paddingTop: 6,
    paddingHorizontal: 22,
    paddingBottom: 20,
  }),
  quitButtonFooter: Styles.createViewStyle({
    paddingTop: 20,
    paddingBottom: 22,
    paddingHorizontal: 22,
  }),
  cellFooterLabel: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    color: colors.white60,
  }),
};
