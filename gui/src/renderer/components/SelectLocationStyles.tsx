import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { ScopeBar } from './ScopeBar';

export const StyledScopeBar = styled(ScopeBar)({
  marginTop: '8px',
});

export default {
  select_location: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  content: Styles.createViewStyle({
    overflow: 'visible',
  }),
  navigationBarAttachment: Styles.createTextStyle({
    marginTop: 8,
    paddingHorizontal: 4,
  }),
  selectedCell: Styles.createViewStyle({
    backgroundColor: colors.green,
  }),
};
