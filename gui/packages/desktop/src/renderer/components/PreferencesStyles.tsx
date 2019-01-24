import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  preferences: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  preferences__container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  preferences__content: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  preferences__separator: Styles.createViewStyle({
    height: 1,
  }),
};
