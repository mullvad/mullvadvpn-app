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
  title_header: Styles.createViewStyle({
    paddingBottom: 0,
  }),
  subtitle_header: Styles.createViewStyle({
    paddingTop: 0,
  }),
  content: Styles.createViewStyle({
    overflow: 'visible',
  }),
};
