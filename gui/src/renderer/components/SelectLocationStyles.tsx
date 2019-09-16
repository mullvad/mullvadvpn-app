import { Styles } from 'reactxp';
import { colors } from '../../config.json';

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
  scopeBar: Styles.createViewStyle({
    marginTop: 8,
  }),
  selectedCell: Styles.createViewStyle({
    backgroundColor: colors.green,
  }),
};
