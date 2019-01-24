import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  layout: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  header: Styles.createViewStyle({
    flex: 0,
  }),
  container: Styles.createViewStyle({
    flex: 1,
    backgroundColor: colors.blue,
    overflow: 'hidden',
  }),
};
