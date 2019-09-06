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
  header: Styles.createViewStyle({
    paddingBottom: 4,
  }),
  stickyHolder: Styles.createViewStyle({
    marginTop: 4,
  }),
  stickyContent: Styles.createViewStyle({
    paddingHorizontal: 12,
    paddingBottom: 8,

    // NavigationBar already adds some spacing
    paddingTop: 0,
  }),
};
