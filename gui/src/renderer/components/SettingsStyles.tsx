import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import * as Cell from './Cell';

export const OutOfTimeSubText = styled(Cell.SubText)((props: { isOutOfTime: boolean }) => ({
  color: props.isOutOfTime ? colors.red : undefined,
}));

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
    height: 24,
    flex: 0,
  }),
  cellFooter: Styles.createViewStyle({
    paddingTop: 8,
    paddingRight: 24,
    paddingBottom: 24,
    paddingLeft: 24,
  }),
  quitButtonFooter: Styles.createViewStyle({
    paddingTop: 24,
    paddingBottom: 19,
    paddingLeft: 24,
    paddingRight: 24,
  }),
  cellFooterLabel: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    letterSpacing: -0.2,
    color: colors.white60,
  }),
};
